/*
use nanoserde::DeJson;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use ureq;

/// The JSON “author_nicks” array is an array of objects. We only care about the name.
#[derive(DeJson, Debug)]
#[allow(dead_code)]
pub struct AuthorNick {
    pub name: String,
    // Other fields (like abbreviation or releaser) are ignored.
}

/// The JSON “credits” array holds more detailed information about a production’s credits.
/// We use this struct to capture the “nick” (an AuthorNick) plus a category and role.
#[derive(DeJson, Debug)]
#[allow(dead_code)]
pub struct Credit {
    pub nick: AuthorNick,
    pub category: String,
    pub role: String,
}

/// The JSON “download_links” array contains objects with a link class and a URL.
#[derive(DeJson, Debug)]
#[allow(dead_code)]
pub struct DownloadLink {
    pub link_class: String,
    pub url: String,
}

/// The JSON “platforms” array gives details for each platform. (We only really care about the name.)
#[derive(DeJson, Debug)]
#[allow(dead_code)]
pub struct Platform {
    pub url: String,
    pub id: u32,
    pub name: String,
}

/// The JSON “screenshots” array has keys like “original_url” which we map into our struct.
/// We use Serde’s rename attribute to change the JSON key into our field name.
#[derive(DeJson, Debug)]
#[allow(dead_code)]
pub struct Screenshot {
    pub original_url: String,
    pub original_width: u32,
    pub original_height: u32,
    pub standard_url: String,
    pub standard_width: u32,
    pub standard_height: u32,
    pub thumbnail_url: String,
    pub thumbnail_width: u32,
    pub thumbnail_height: u32,
}

/// The main ProductionEntry struct gathers the fields we care about from the JSON.
#[derive(DeJson, Debug)]
pub struct ProductionEntry {
    pub title: String,
    pub release_date: String,
    pub author_nicks: Vec<AuthorNick>,
    pub credits: Vec<Credit>,
    pub download_links: Vec<DownloadLink>,
    pub platforms: Vec<Platform>,
    pub screenshots: Vec<Screenshot>,
    pub tags: Vec<String>,
}

fn parse_json(json_data: &str) -> ProductionEntry {
    DeJson::deserialize_json(json_data).expect("Failed to parse JSON")
}

/// Directory to store cached images
#[allow(dead_code)]
const CACHE_DIR: &str = "image_cache";

/// Computes a SHA256 hash of the URL to use as a unique filename
#[allow(dead_code)]
fn hash_url(url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(url);
    format!("{:x}", hasher.finalize())
}

/// Extracts the file extension from the URL or defaults to "bin" if not found
#[allow(dead_code)]
fn get_extension(url: &str) -> &str {
    Path::new(url)
        .extension()
        .and_then(|ext| ext.to_str())
        .filter(|ext| !ext.is_empty()) // Ensure we don't return an empty extension
        .unwrap_or("bin") // Default to "bin" if no valid extension found
}

/// Checks if the image is already cached; otherwise, downloads it
#[allow(dead_code)]
pub fn get_image(url: &str) -> io::Result<String> {
    // Ensure the cache directory exists
    fs::create_dir_all(CACHE_DIR)?;

    // Extract file extension from URL
    let file_extension = get_extension(url);

    // Hash the URL to get a unique filename
    let file_name = format!("{}.{}", hash_url(url), file_extension);
    let file_path = PathBuf::from(CACHE_DIR).join(&file_name);

    // Check if the image is already cached
    if file_path.exists() {
        println!("Cache hit: {}", file_path.display());
        return Ok(file_path.to_string_lossy().to_string());
    }

    // Fetch the image from the URL
    println!("Downloading: {}", url);
    let resp = ureq::get(url)
        .call()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let mut reader = resp
        .into_body()
        .into_with_config()
        .reader();

    // Read binary data from response using `response.into_reader()`
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;  // Changed to use `reader()`

    // Write the image to the cache directory
    let mut file = File::create(&file_path)?;
    file.write_all(&bytes)?;

    println!("Saved to cache: {}", file_path.display());
    Ok(file_path.to_string_lossy().to_string())
}

/// Searches for a file by checking the current and parent directories recursively.
/// Once found, it returns the absolute path of the file.
fn find_file_upwards(filename: &str) -> Option<PathBuf> {
    let mut current_dir = std::env::current_dir().ok()?;

    loop {
        let potential_path = current_dir.join(filename);
        if potential_path.exists() {
            return Some(potential_path);
        }

        if !current_dir.pop() { // Moves up one directory level
            break;
        }
    }
    None
}

fn load_file(filename: &str) -> io::Result<String> {
    if let Some(file_path) = find_file_upwards(filename) {
        fs::read_to_string(file_path)
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("File '{}' not found in any parent directory.", filename),
        ))
    }
}

// TODO: This should be async on separate thread
pub fn get_demo_entry_by_file(url: &str) -> ProductionEntry {
    load_file(url)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
        .and_then(|data| {
            let entry = parse_json(&data);
            Ok(entry)
        })
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_production_entry() {
        let data = std::fs::read_to_string("../../crates/demozoo-fetcher/test-data/2.json")
            .expect("Unable to read file");

        let entry: ProductionEntry =
            DeJson::deserialize_json(&data).expect("Failed to deserialize JSON");

        // Verify that the basic fields were deserialized correctly.
        assert_eq!(entry.title, "State of the Art");
        assert_eq!(entry.release_date, "1992-12-29");

        // The author_nicks array should contain one entry with the name "Spaceballs".
        assert_eq!(entry.author_nicks.len(), 1);
        assert_eq!(entry.author_nicks[0].name, "Spaceballs");

        // There should be 4 credit entries; verify that one of them is for Music ("Travolta").
        assert_eq!(entry.credits.len(), 4);
        let music_credit = entry
            .credits
            .iter()
            .find(|credit| credit.category == "Music")
            .expect("Missing music credit");
        assert_eq!(music_credit.nick.name, "Travolta");

        // There should be 4 download links.
        assert_eq!(entry.download_links.len(), 4);

        // The platforms array should have one platform with name "Amiga OCS/ECS".
        assert_eq!(entry.platforms.len(), 1);
        assert_eq!(entry.platforms[0].name, "Amiga OCS/ECS");

        // According to the sample JSON, there should be 23 screenshots.
        assert_eq!(entry.screenshots.len(), 23);

        // And there should be 3 tags.
        assert_eq!(entry.tags.len(), 3);
    }

    #[test]
    fn test_valid_extension() {
        assert_eq!(get_extension("file.txt"), "txt");
        assert_eq!(get_extension("archive.tar.gz"), "gz");
        assert_eq!(get_extension("/path/to/some/file.rs"), "rs");
        assert_eq!(get_extension("C:\\Users\\user\\document.pdf"), "pdf");
    }

    #[test]
    fn test_no_extension() {
        assert_eq!(get_extension("file"), "bin");
        assert_eq!(get_extension("/path/to/some/file_without_ext"), "bin");
        assert_eq!(get_extension("C:\\Users\\user\\document"), "bin");
    }

    #[test]
    fn test_empty_extension() {
        assert_eq!(get_extension("file."), "bin");
        assert_eq!(get_extension("/path/to/some/file."), "bin");
    }

    #[test]
    fn test_hidden_files() {
        assert_eq!(get_extension(".hiddenfile"), "bin");
        assert_eq!(get_extension("/path/.config"), "bin");
        assert_eq!(get_extension(".gitignore"), "bin");
    }

    #[test]
    fn test_weird_cases() {
        assert_eq!(get_extension("..."), "bin");
        assert_eq!(get_extension("file.name.with.dots.log"), "log");
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(get_extension(""), "bin");
    }
}

 */
