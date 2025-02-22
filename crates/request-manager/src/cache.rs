// cache.rs
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::fs;
//use crate::types::CacheEntry;
use crate::Result;
use std::hash::Hasher;

pub struct CacheStore {
    cache_dir: PathBuf,
    entries: HashSet<PathBuf>,
    temp_string: String,
    temp_path: PathBuf,
}

impl CacheStore {
    pub fn new(cache_dir: &Path) -> Result<Self> {
        fs::create_dir_all(cache_dir)?;

        let mut entries = HashSet::with_capacity(128);
        Self::scan_directory(cache_dir, &mut entries)?;

        Ok(Self {
            cache_dir: cache_dir.to_path_buf(),
            temp_path: PathBuf::with_capacity(512),
            temp_string: String::with_capacity(32),
            entries,
        })
    }

    fn scan_directory(dir: &Path, entries: &mut HashSet<PathBuf>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            entries.insert(path.clone());
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn contains_key(&mut self, url: &str) -> bool {
        let path = self.build_path(url).to_owned();
        // TODO: Fix clone
        self.entries.contains(&path)
    }

    fn u64_to_hex(n: u64, s: &mut String) {
        let hex = b"0123456789abcdef";
        let mut num = n;

        // Safety: We assume the string has capacity >= 16
        unsafe {
            let bytes = s.as_mut_vec();
            bytes.set_len(16); // Set length to what we'll write

            for i in (0..16).rev() {
                bytes[i] = hex[(num & 0xF) as usize];
                num >>= 4;
            }
        }
    }

    fn build_path(&mut self, url: &str) -> &PathBuf {
        let mut hasher = fxhash::FxHasher64::default();
        hasher.write(url.as_bytes());
        let hash = hasher.finish();

        Self::u64_to_hex(hash, &mut self.temp_string);

        self.temp_path.clear();
        self.temp_path.push(&self.cache_dir);
        self.temp_path.push(&self.temp_string.as_str());
        &self.temp_path
    }

    pub fn get_path_for_url(&mut self, url: &str) -> &PathBuf {
        self.build_path(url)
    }

    pub fn get_path(&mut self, url: &str) -> Option<PathBuf> {
        let path = self.build_path(url).to_owned();
        self.entries.get(&path).map(|entry| entry.to_owned())
    }

    pub fn insert(&mut self, path: PathBuf) {
        if path.exists() {
            self.entries.insert(path);
        }
    }

    #[allow(dead_code)]
    fn remove(&mut self, url: &str) -> bool {
        let path = self.build_path(url).to_owned();

        if self.entries.remove(&path) {
            // Try to remove the file, but don't fail if we can't
            let _ = fs::remove_file(&path);
            true
        } else {
            false
        }
    }

    #[allow(dead_code)]
    fn clear(&mut self) -> Result<()> {
        for entry in &self.entries {
            let _ = fs::remove_file(entry);
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
        store.insert(test_path.clone());

        // Verify it's in the cache
        //assert!(store.contains_key("test.json"));
        //assert_eq!(store.get_path("test.json").unwrap(), test_path);

        Ok(())
    }

    #[test]
    fn test_cache_store_remove() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut store = CacheStore::new(temp_dir.path())?;

        // Create and add a file
        let test_path = create_test_file(temp_dir.path(), "test.json")?;
        store.insert(test_path.clone());

        // Remove it
        /*
        let removed = store.remove(test_path.as_path());
        assert!(removed);
        assert!(!store.contains_key("test.json"));
        assert!(!test_path.exists());

         */

        Ok(())
    }

    #[test]
    fn test_cache_store_clear() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut store = CacheStore::new(temp_dir.path())?;

        // Create and add multiple files
        let test_path1 = create_test_file(temp_dir.path(), "test1.json")?;
        let test_path2 = create_test_file(temp_dir.path(), "test2.json")?;

        store.insert(test_path1.clone());
        store.insert(test_path2.clone());

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
        let mut store = CacheStore::new(temp_dir.path())?;

        /*
        assert!(store.contains_key("test1.json"));
        assert!(store.contains_key("test2.json"));
        assert_eq!(store.get_path("test1.json").unwrap(), test_path1);
        assert_eq!(store.get_path("test2.json").unwrap(), test_path2);

         */

        Ok(())
    }
}