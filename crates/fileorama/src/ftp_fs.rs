use crate::{Driver, DriverType, Error, FilesDirs, LoadStatus, Progress};
use log::error;
use std::fmt::{Debug, Formatter};
use suppaftp::{FtpError, FtpStream};

// This is kinda ugly, but better than testing non-supported paths on a remote server
#[cfg(target_os = "windows")]
pub const FTP_URL: &str = "ftp:\\";

#[cfg(not(target_os = "windows"))]
pub const FTP_URL: &str = "ftp:/";

pub struct FtpFs {
    data: Option<FtpStream>,
}

impl Debug for FtpFs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FtpFs").finish()
    }
}

impl FtpFs {
    pub fn new() -> FtpFs {
        FtpFs { data: None }
    }
}

impl FtpFs {
    fn find_server_name(url: &str) -> Option<&str> {
        let stripped_url = url.strip_prefix(FTP_URL).unwrap_or(url);
        stripped_url.splitn(2, '/').next().filter(|s| !s.is_empty())
    }
}

/// Extracts the file size from an FTP directory listing entry.
///
/// # Arguments
/// * `entry` - The FTP directory listing entry to parse.
///
/// # Returns
/// A `Result<usize, Error>` containing the file size if successful, otherwise an error.
fn extract_file_size(entry: &str) -> Result<usize, Error> {
    let parts: Vec<&str> = entry.split_whitespace().collect();
    if parts.len() < 5 {
        return Err(Error::InvalidEntry(entry.to_owned()));
    }
    parts[4]
        .parse::<usize>()
        .map_err(|_| Error::InvalidSize(parts[4].to_owned()))
}

/// Parses an FTP directory listing entry to determine if it is a file or directory.
///
/// # Arguments
/// * `entry` - The FTP directory listing entry to parse.
///
/// # Returns
/// A tuple containing a boolean indicating if it's a directory and the name of the file/directory.
fn parse_ftp_entry(entry: &str) -> Option<(String, bool)> {
    // As the result from the FTP server is given in this format we have to split the string and pick out the data we want
    // -rw-rw-r--    1 1001       1001          5046034 May 25 16:00 allmods.zip
    // drwxrwxr-x    7 1001       1001             4096 Jan 20  2018 incoming

    let parts: Vec<&str> = entry.split_whitespace().collect();
    // Ensure we have enough parts (index 8 should be the filename)
    if parts.len() < 9 {
        return None;
    }

    let is_directory = parts[0].starts_with('d');
    let name = parts[8].to_owned();

    Some((name, is_directory))
}

impl Driver for FtpFs {
    /// This indicates that the file system is remote (such as ftp, https) and has no local path
    fn is_remote(&self) -> bool {
        true
    }

    fn create_from_data(
        &self,
        _data: Box<[u8]>,
        _file_ext_hint: &str,
        _driver_data: &Option<Box<[u8]>>,
    ) -> Option<DriverType> {
        None
    }

    fn can_create_from_data(&self, _data: &[u8], _file_ext_hint: &str) -> bool {
        false
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
    fn create_instance(&self) -> DriverType {
        Box::new(FtpFs::new())
    }

    /// Used when creating an instance of the driver with a path to load from
    fn create_from_url(&self, url: &str) -> Option<DriverType> {
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
            stream
                .transfer_type(suppaftp::types::FileType::Binary)
                .unwrap();

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
            let file_size = extract_file_size(&dirs_and_files[0])?;

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
                        .map_err(|op| FtpError::SecureError(op.to_string()))?;
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
            if let Some((name, is_directory)) = parse_ftp_entry(&dir_file) {
                if is_directory {
                    dirs.push(name);
                } else {
                    files.push(name);
                }
            }
        }

        files.sort();
        dirs.sort();

        progress.step()?;

        Ok(FilesDirs::new(files, dirs))
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_server_name_with_full_url() {
        let url = "ftp://example.com/path/to/file";
        assert_eq!(FtpFs::find_server_name(url), Some("example.com"));
    }

    #[test]
    fn test_find_server_name_with_no_path() {
        let url = "ftp://example.com";
        assert_eq!(FtpFs::find_server_name(url), Some("example.com"));
    }

    #[test]
    fn test_find_server_name_without_prefix() {
        let url = "example.com/path/to/file";
        assert_eq!(FtpFs::find_server_name(url), Some("example.com"));
    }

    #[test]
    fn test_find_server_name_only_server_name() {
        let url = "example.com";
        assert_eq!(FtpFs::find_server_name(url), Some("example.com"));
    }

    #[test]
    fn test_find_server_name_with_trailing_slash() {
        let url = "ftp://example.com/";
        assert_eq!(FtpFs::find_server_name(url), Some("example.com"));
    }

    #[test]
    fn test_find_server_name_no_separator_after_prefix() {
        let url = "ftp://";
        assert_eq!(FtpFs::find_server_name(url), None);
    }

    #[test]
    fn test_find_server_name_empty_string() {
        let url = "";
        assert_eq!(FtpFs::find_server_name(url), Some(""));
    }

    #[test]
    fn test_find_server_name_only_slash() {
        let url = "/";
        assert_eq!(FtpFs::find_server_name(url), Some(""));
    }

    #[test]
    fn test_find_server_name_no_prefix_with_slash() {
        let url = "/path/to/file";
        assert_eq!(FtpFs::find_server_name(url), Some(""));
    }

    #[test]
    fn test_parse_ftp_entry_directory() {
        let entry = "drwxrwxr-x    7 1001       1001             4096 Jan 20  2018 incoming";
        assert_eq!(parse_ftp_entry(entry), Some(("incoming".to_owned(), true)));
    }

    #[test]
    fn test_parse_ftp_entry_file() {
        let entry = "-rw-rw-r--    1 1001       1001          5046034 May 25 16:00 allmods.zip";
        assert_eq!(parse_ftp_entry(entry), Some(("allmods.zip".to_owned(), false)));
    }

    #[test]
    fn test_parse_ftp_entry_invalid_format() {
        let entry = "-rw-rw-r--    1 1001       1001          5046034";
        assert_eq!(parse_ftp_entry(entry), None);  // Not enough parts
    }

    #[test]
    fn test_parse_ftp_entry_empty_string() {
        let entry = "";
        assert_eq!(parse_ftp_entry(entry), None);  // Empty input
    }
}
*/
