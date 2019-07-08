use std::collections::HashMap;
use std::path::Path;

use crate::error::Result;

/// The struct to hold key value pairs.
/// Currently it uses memory storage.
pub struct KvStore {
    store: HashMap<String, String>,
}

/// A store that keeps key-value pairs in memory
/// ```rust
/// # use kvs::KvStore;
/// let mut store = KvStore::new();
///
/// store.set("key1".to_owned(), "value1".to_owned());
/// assert_eq!(store.get("key1".to_owned()), Some("value1".to_owned()));
///
/// store.remove("key1".to_owned());
/// assert_eq!(store.get("key1".to_owned()), None);
/// ```
impl KvStore {
    /// Creat a key value store
    pub fn new() -> KvStore {
        KvStore {
            store: HashMap::new(),
        }
    }

    /// Open logfile
    pub fn open(path: &Path) -> Result<KvStore> {
        panic!();
    }

    /// Set key value to store
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        //self.store.insert(key, value);
        panic!();
    }

    /// Get value by a key from store
    pub fn get(&self, key: String) -> Result<Option<String>> {
        //self.store.get(&key).cloned()
        panic!();
    }

    /// Remove key value from store
    pub fn remove(&mut self, key: String) -> Result<()> {
        //self.store.remove(&key);
        panic!();
    }
}
