use crate::{Error, FilesDirs, LoadStatus, MemoryDriver, MemoryDriverType, Progress};
use std::borrow::Cow;
use std::collections::HashSet;
use std::io::{Cursor, Read};
use zip::ZipArchive;

// This is kinda ugly, but better than testing non-supported paths on a remote server
#[cfg(target_os = "windows")]
pub const FTP_CHECK: &str = "ftp:\\";

#[cfg(not(target_os = "windows"))]
pub const FTP_CHECK: &str = "ftp:/";

#[cfg(not(test))]
use log::{error, trace};

#[cfg(test)]
use std::{println as trace, println as error};

#[derive(Debug)]
enum ZipInternal {
    MemReader(ZipArchive<Cursor<Box<[u8]>>>),
    None,
}

#[derive(Debug)]
pub struct ZipFs {
    data: ZipInternal,
}

impl ZipFs {
    pub fn new() -> ZipFs {
        ZipFs {
            data: ZipInternal::None,
        }
    }

    fn get_dirs(
        path: &str,
        progress: &mut Progress,
        filenames: &mut dyn Iterator<Item = &str>,
    ) -> Result<FilesDirs, Error> {
        trace!("ZIP get_dirs: {}", path);

        let mut paths = HashSet::<String>::new();
        let mut files = Vec::with_capacity(256);

        let match_dir: String = if path.ends_with('/') {
            path.into()
        } else if path.is_empty() {
            "".into()
        } else {
            format!("{}/", path)
        };

        let dir_len = match_dir.len();

        for p in filenames {
            if !p.starts_with(&match_dir) {
                continue;
            }

            let t = &p[dir_len..];

            if let Some(pos) = t.find('/') {
                if pos <= t.len() {
                    paths.insert(t[..pos].to_owned());
                }
            } else {
                files.push(t.to_owned());
            }
        }

        progress.set_step(3);

        progress.step()?;

        let mut dirs = Vec::with_capacity(paths.len());

        for s in paths {
            if s.is_empty() {
                continue;
            }

            dirs.push(s.to_owned());
        }

        progress.step()?;

        files.sort();
        dirs.sort();

        progress.step()?;

        Ok(FilesDirs::new(files, dirs))
    }
}

impl MemoryDriver for ZipFs {
    fn name(&self) -> &'static str {
        "zip_fs"
    }

    // Create a new instance given data. The Driver will take ownership of the data
    fn create_instance(&self) -> MemoryDriverType {
        Box::new(ZipFs::new())
    }

    // Get some data in and returns true if driver can be mounted from it
    fn can_create_from_data(&self, data: &[u8]) -> bool {
        let c = std::io::Cursor::new(data);
        ZipArchive::new(c).is_ok()
    }

    // Create a new instance given data. The Driver will take ownership of the data
    fn create_from_data(&self, data: Box<[u8]>) -> Option<MemoryDriverType> {
        let a = match ZipArchive::new(std::io::Cursor::new(data)) {
            Ok(a) => a,
            Err(e) => {
                error!("ZipFs Error: {:}", e);
                return None;
            }
        };

        Some(Box::new(ZipFs {
            data: ZipInternal::MemReader(a),
        }))
    }

    /// Returns a handle which updates the progress and returns the loaded data. This will try to
    fn load(&mut self, local_path: &str, progress: &mut Progress) -> Result<LoadStatus, Error> {
        if local_path.is_empty() {
            return Ok(LoadStatus::Directory);
        }

        // The zip loader doesn't support \ for internal paths so replace with /
        let local_path: Cow<str> = if !local_path.contains('\\') {
            local_path.into()
        } else {
            local_path.replace('\\', "/").into()
        };

        let read_file = match &mut self.data {
            ZipInternal::MemReader(a) => a.by_name(&local_path),
            ZipInternal::None => return Ok(LoadStatus::NotFound),
        };

        let mut file = match read_file {
            Err(_) => {
                trace!("file not found: {}", local_path);
                return Ok(LoadStatus::NotFound);
            }
            Ok(f) => f,
        };

        if file.is_dir() {
            return Ok(LoadStatus::Directory);
        }

        let len = file.size() as usize;
        let mut output_data = vec![0u8; len];

        // if file is small than 10k we just unpack it directly without progress
        if len < 10 * 1024 {
            progress.set_step(1);
            file.read_to_end(&mut output_data)?;
            progress.step()?;
        } else {
            // above 10k we read in 10 chunks
            let loop_count = 10;
            let block_len = len / loop_count;

            progress.set_step(loop_count);

            for i in 0..loop_count + 1 {
                let block_offset = i * block_len;
                let read_amount = usize::min(len - block_offset, block_len);
                file.read_exact(&mut output_data[block_offset..block_offset + read_amount])?;
                progress.step()?;
            }
        }

        Ok(LoadStatus::Data(output_data.into_boxed_slice()))
    }

    fn get_directory_list(
        &mut self,
        local_path: &str,
        progress: &mut Progress,
    ) -> Result<FilesDirs, Error> {
        match &self.data {
            ZipInternal::MemReader(a) => Self::get_dirs(local_path, progress, &mut a.file_names()),
            ZipInternal::None => Ok(FilesDirs::default()),
        }
    }
}
