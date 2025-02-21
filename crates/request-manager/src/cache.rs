// cache.rs
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use crate::types::CacheEntry;
use crate::Result;

pub struct CacheStore {
    cache_dir: PathBuf,
    entries: HashMap<String, CacheEntry>,
}

impl CacheStore {
    pub fn new(cache_dir: &Path) -> Result<Self> {
        fs::create_dir_all(cache_dir)?;

        let mut entries = HashMap::new();
        Self::scan_directory(cache_dir, &mut entries)?;

        Ok(Self {
            cache_dir: cache_dir.to_path_buf(),
            entries,
        })
    }

    fn scan_directory(dir: &Path, entries: &mut HashMap<String, CacheEntry>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            if let Some(file_name) = path.file_name() {
                if let Some(url) = file_name.to_str() {
                    entries.insert(
                        url.to_string(),
                        CacheEntry {
                            path: path.clone(),
                        }
                    );
                }
            }
        }
        Ok(())
    }

    pub fn contains_key(&self, url: &str) -> bool {
        self.entries.contains_key(url)
    }

    pub fn get_path(&self, url: &str) -> Option<PathBuf> {
        self.entries.get(url).map(|entry| entry.path.clone())
    }

    pub fn insert(&mut self, url: String, path: PathBuf) -> Result<()> {
        // Ensure the file exists
        if !path.exists() {
            return Ok(());
        }

        let metadata = fs::metadata(&path)?;

        self.entries.insert(
            url,
            CacheEntry {
                path,
            },
        );

        Ok(())
    }

    fn remove(&mut self, url: &str) -> Option<CacheEntry> {
        if let Some(entry) = self.entries.remove(url) {
            // Try to remove the file, but don't fail if we can't
            let _ = fs::remove_file(&entry.path);
            Some(entry)
        } else {
            None
        }
    }

    fn clear(&mut self) -> Result<()> {
        for entry in self.entries.values() {
            let _ = fs::remove_file(&entry.path);
        }
        self.entries.clear();
        fs::remove_dir_all(&self.cache_dir)?;
        fs::create_dir_all(&self.cache_dir)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    fn create_test_file(dir: &Path, name: &str) -> Result<PathBuf> {
        let path = dir.join(name);
        File::create(&path)?;
        Ok(path)
    }

    #[test]
    fn test_cache_store_basic() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut store = CacheStore::new(temp_dir.path())?;

        // Create a test file
        let test_path = create_test_file(temp_dir.path(), "test.json")?;

        // Add to cache
        store.insert("test.json".to_string(), test_path.clone())?;

        // Verify it's in the cache
        assert!(store.contains_key("test.json"));
        assert_eq!(store.get_path("test.json").unwrap(), test_path);

        Ok(())
    }

    #[test]
    fn test_cache_store_remove() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut store = CacheStore::new(temp_dir.path())?;

        // Create and add a file
        let test_path = create_test_file(temp_dir.path(), "test.json")?;
        store.insert("test.json".to_string(), test_path.clone())?;

        // Remove it
        let removed = store.remove("test.json");
        assert!(removed.is_some());
        assert!(!store.contains_key("test.json"));
        assert!(!test_path.exists());

        Ok(())
    }

    #[test]
    fn test_cache_store_clear() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut store = CacheStore::new(temp_dir.path())?;

        // Create and add multiple files
        let test_path1 = create_test_file(temp_dir.path(), "test1.json")?;
        let test_path2 = create_test_file(temp_dir.path(), "test2.json")?;

        store.insert("test1.json".to_string(), test_path1.clone())?;
        store.insert("test2.json".to_string(), test_path2.clone())?;

        // Clear cache
        store.clear()?;

        assert!(!store.contains_key("test1.json"));
        assert!(!store.contains_key("test2.json"));
        assert!(!test_path1.exists());
        assert!(!test_path2.exists());

        Ok(())
    }

    #[test]
    fn test_cache_store_scan() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create some files before initializing store
        let test_path1 = create_test_file(temp_dir.path(), "test1.json")?;
        let test_path2 = create_test_file(temp_dir.path(), "test2.json")?;

        // Initialize store - should find existing files
        let store = CacheStore::new(temp_dir.path())?;

        assert!(store.contains_key("test1.json"));
        assert!(store.contains_key("test2.json"));
        assert_eq!(store.get_path("test1.json").unwrap(), test_path1);
        assert_eq!(store.get_path("test2.json").unwrap(), test_path2);

        Ok(())
    }
}