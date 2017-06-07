use std::sync::Arc;

pub trait KvStore {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>>;
}

impl KvStore for Arc<KvStore + Send + Sync> {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        (**self).get(key)
    }
}

impl KvStore for Box<KvStore> {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        (**self).get(key)
    }
}

pub mod cdb;
pub mod mtbl;
