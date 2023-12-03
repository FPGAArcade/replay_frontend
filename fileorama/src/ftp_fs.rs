use crate::{Error, FilesDirs, IoDriver, IoDriverType, LoadStatus, Progress};
use ftp::{FtpError, FtpStream};
use log::error;
use std::path::MAIN_SEPARATOR;

// This is kinda ugly, but better than testing non-supported paths on a remote server
#[cfg(target_os = "windows")]
pub const FTP_URL: &str = "ftp:\\";

#[cfg(not(target_os = "windows"))]
pub const FTP_URL: &str = "ftp:/";

#[derive(Debug)]
pub struct FtpFs {
    data: Option<FtpStream>,
}

impl FtpFs {
    pub fn new() -> FtpFs {
        FtpFs { data: None }
    }
}

impl FtpFs {
    fn find_server_name(url: &str) -> Option<&str> {
        if let Some(url) = url.strip_prefix(FTP_URL) {
            // handle if we have name and no path
            if !url.contains(MAIN_SEPARATOR) {
                return Some(url);
            }

            if let Some(offset) = url.find(MAIN_SEPARATOR) {
                return Some(&url[..offset]);
            }
        } else {
            if !url.contains(MAIN_SEPARATOR) {
                return Some(url);
            }

            if let Some(offset) = url.find(MAIN_SEPARATOR) {
                return Some(&url[..offset]);
            }
        }

        None
    }
}

impl IoDriver for FtpFs {
    /// This indicates that the file system is remote (such as ftp, https) and has no local path
    fn is_remote(&self) -> bool {
        true
    }

    fn name(&self) -> &'static str {
        "ftp_fs"
    }

    /// If the driver supports a certain url
    fn supports_url(&self, url: &str) -> bool {
        // Only supports urls that starts with ftp
        url.starts_with(FTP_URL) || url.starts_with("ftp.")
    }

    // Create a new instance given data. The Driver will take ownership of the data
    fn create_instance(&self) -> IoDriverType {
        Box::new(FtpFs::new())
    }

    /*
    // Get some data in and returns true if driver can be mounted from it
    fn can_load_from_url(&self, url: &str) -> bool {
        // Only supports urls that starts with ftp
        if !url.starts_with(FTP_URL) && !url.starts_with("ftp.") {
            return false;
        }

        // Make sure we don't have any slashes in the path except ftp start
        if let Some(url) = url.strip_prefix(FTP_URL) {
            !url.contains(MAIN_SEPARATOR)
        } else {
            !url.contains(MAIN_SEPARATOR)
        }
    }
    */

    /// Used when creating an instance of the driver with a path to load from
    fn create_from_url(&self, url: &str) -> Option<IoDriverType> {
        if let Some(url) = Self::find_server_name(url) {
            let url_with_port = if url.contains(':') {
                url.to_owned()
            } else {
                format!("{}:21", url)
            };

            let mut stream = match FtpStream::connect(url_with_port) {
                Ok(stream) => stream,
                Err(e) => {
                    error!("Unable to connect to {:?}", e);
                    return None;
                }
            };

            stream.login("anonymous", "anonymous").unwrap();
            stream.transfer_type(ftp::types::FileType::Binary).unwrap();

            return Some(Box::new(FtpFs { data: Some(stream) }));
        }

        None
    }

    /// Returns a handle which updates the progress and returns the loaded data. This will try to
    fn load(&mut self, path: &str, progress: &mut Progress) -> Result<LoadStatus, Error> {
        let conn = self.data.as_mut().unwrap();

        // We get a listing of the files first here because if we try to do 'SIZE' on a directory
        // this command will hang, if this is a fault of the FTP server or ftp-rs I don't know, but this is a workaround at least
        let dirs_and_files = conn.list(Some(path))?;

        // To validate that we are only checking one file we expect the listing command to return one entry
        // with the first file flag not being set to 'd'
        if dirs_and_files.len() == 1 && !dirs_and_files[0].starts_with('d') {
            // split up the size so we can access the size
            let t = dirs_and_files[0].split_whitespace().collect::<Vec<&str>>();
            let file_size = t[4].parse::<usize>()?;

            let block_len = 64 * 1024;
            let loop_count = file_size / block_len;
            progress.set_step(loop_count);

            let output_data = conn.retr(path, |reader| {
                let mut output_data = vec![0u8; file_size];
                let mut pro = progress.clone();

                for i in 0..loop_count + 1 {
                    let block_offset = i * block_len;
                    let read_amount = usize::min(file_size - block_offset, block_len);
                    reader
                        .read_exact(&mut output_data[block_offset..block_offset + read_amount])
                        .map_err(FtpError::ConnectionError)?;
                    pro.step()
                        .map_err(|op| FtpError::InvalidResponse(op.to_string()))?;
                }

                Ok(output_data)
            })?;

            Ok(LoadStatus::Data(output_data.into_boxed_slice()))
        } else {
            // if we didn't get any size here we assume it's a directory.
            Ok(LoadStatus::Directory)
        }
    }

    fn get_directory_list(
        &mut self,
        path: &str,
        progress: &mut Progress,
    ) -> Result<FilesDirs, Error> {
        let conn = self.data.as_mut().unwrap();

        progress.set_step(2);

        let dirs_and_files = conn.list(Some(path))?;

        let mut dirs = Vec::with_capacity(dirs_and_files.len());
        let mut files = Vec::with_capacity(dirs_and_files.len());

        progress.step()?;

        for dir_file in dirs_and_files {
            // As the result from the FTP server is given in this format we have to split the string and pick out the data we want
            // -rw-rw-r--    1 1001       1001          5046034 May 25 16:00 allmods.zip
            // drwxrwxr-x    7 1001       1001             4096 Jan 20  2018 incoming
            let t = dir_file.split_whitespace().collect::<Vec<&str>>();

            // if flags starts with 'd' we assume it's a directory
            if t[0].starts_with('d') {
                dirs.push(t[8].to_owned());
            } else {
                files.push(t[8].to_owned());
            }
        }

        files.sort();
        dirs.sort();

        progress.step()?;

        Ok(FilesDirs::new(files, dirs))
    }
}
