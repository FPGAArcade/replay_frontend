// cache.rs
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
//use crate::types::CacheEntry;
use std::hash::Hasher;

pub struct CacheStore {
    cache_dir: PathBuf,
    entries: HashSet<PathBuf>,
    //temp_string: String,
    //temp_path: PathBuf,
}

impl CacheStore {
    pub fn new(cache_dir: &str) -> std::io::Result<Self> {
        fs::create_dir_all(cache_dir)?;

        let mut entries = HashSet::with_capacity(128);
        Self::scan_directory(cache_dir, &mut entries)?;

        Ok(Self {
            cache_dir: Path::new(cache_dir).to_path_buf(),
            //temp_path: PathBuf::with_capacity(512),
            //temp_string: String::with_capacity(32),
            entries,
        })
    }

    fn scan_directory(dir: &str, entries: &mut HashSet<PathBuf>) -> std::io::Result<()> {
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
        let mut cache_path = PathBuf::with_capacity(128);
        Self::get_cache_path(url, &self.cache_dir, &mut cache_path);
        // TODO: Fix clone
        self.entries.contains(&cache_path)
    }

    fn u64_to_hex(n: u64, output: &mut [u8; 16]) {
        let hex = b"0123456789abcdef";
        let mut num = n;

        for i in (0..16).rev() {
            output[i] = hex[(num & 0xF) as usize];
            num >>= 4;
        }
    }

    pub fn get_cache_path<P>(url: &str, dir: P, output: &mut PathBuf)
    where
        P: AsRef<Path>,
    {
        let mut hex_string_buffer = [0u8; 16];
        let mut hasher = fxhash::FxHasher64::default();
        hasher.write(url.as_bytes());
        let hash = hasher.finish();

        Self::u64_to_hex(hash, &mut hex_string_buffer);

        output.clear();
        output.push(dir.as_ref());
        output.push(unsafe { std::str::from_utf8_unchecked(&hex_string_buffer) });
    }

    /*
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

     */

    /*
    pub fn get_path_for_url(&mut self, url: &str) -> &PathBuf {
        let mut cache_path = PathBuf::with_capacity(128);
        Self::get_cache_path(url, &self.cache_dir, &mut cache_path);
    }

     */

    /*
    pub fn get_path(&mut self, url: &str) -> Option<PathBuf> {
        let mut path = PathBuf::with_capacity(128);
        Self::get_cache_path(url, &self.cache_dir, &mut path);
        self.entries.get(&path).map(|entry| entry.to_owned())
    }

     */

    /*
    pub fn insert(&mut self, path: PathBuf) {
        if path.exists() {
            self.entries.insert(path);
        }
    }

     */

    #[allow(dead_code)]
    fn remove(&mut self, url: &str) -> bool {
        let mut path = PathBuf::with_capacity(128);
        Self::get_cache_path(url, &self.cache_dir, &mut path);

        if self.entries.remove(&path) {
            // Try to remove the file, but don't fail if we can't
            // TODO: We shouldn't do this here because it may stall the main-thread
            let _ = fs::remove_file(&path);
            true
        } else {
            false
        }
    }

    #[allow(dead_code)]
    fn clear(&mut self) -> std::io::Result<()> {
        for entry in &self.entries {
            let _ = fs::remove_file(entry);
        }

        self.entries.clear();
        fs::remove_dir_all(&self.cache_dir)?;
        fs::create_dir_all(&self.cache_dir)?;
        Ok(())
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    fn create_test_file(dir: &Path, name: &str) -> std::io::Result<PathBuf> {
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

 */
