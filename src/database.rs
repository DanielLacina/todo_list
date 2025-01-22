pub trait Database: Send + Sync + 'static + Default {
    fn get(&self, key: &str) -> Option<Vec<u8>>;
    fn insert(&self, key: &str, value: &str);
    fn delete(&self, key: &str);
}

use dashmap::DashMap;

#[derive(Default)]
pub struct MockDatabase {
    db: DashMap<String, Vec<u8>>,
}

impl Database for MockDatabase {
    fn get(&self, key: &str) -> Option<Vec<u8>> {
        match self.db.get(key) {
            Some(value) => Some(value.clone()),
            _ => None,
        }
    }
    fn insert(&self, key: &str, value: &str) {
        self.db
            .insert(key.to_string(), value.to_string().into_bytes());
    }
    fn delete(&self, key: &str) {
        self.db.remove(key);
    }
}
