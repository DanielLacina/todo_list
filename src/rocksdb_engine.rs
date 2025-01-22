use crate::database::Database;
use rocksdb::{Options, DB};
use std::sync::Arc;

pub struct RocksDBEngine {
    pub db: Arc<DB>,
}

impl RocksDBEngine {
    pub fn new() -> Self {
        let path = "todo_list";
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, path).unwrap();
        Self { db: Arc::new(db) }
    }
}

impl Database for RocksDBEngine {
    fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.db.get(key).unwrap()
    }
    fn insert(&self, key: &str, value: &str) {
        self.db.put(key, value);
    }

    fn delete(&self, key: &str) {
        self.db.delete(key);
    }
}

impl Default for RocksDBEngine {
    fn default() -> Self {
        Self::new()
    }
}
