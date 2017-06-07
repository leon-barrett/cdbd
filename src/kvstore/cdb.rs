use super::KvStore;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use objpool::Pool;
use tinycdb::Cdb;

pub type CdbPool = Arc<Pool<Box<Cdb>>>;

pub fn new_cdb_pool(p: &Path, pool_size: usize) -> CdbPool {
    let p = PathBuf::from(p);
    let pool = Pool::with_capacity(pool_size, move || Cdb::open(&p).unwrap());
    // Warm up the pool.
    (0..pool_size).map(|_: usize| (*pool).get()).collect::<Vec<_>>();
    pool
}

impl KvStore for CdbPool {
    fn get(self: &CdbPool, key: &[u8]) -> Option<Vec<u8>> {
        Pool::get(&*self).find(&key).map(Vec::from)
    }
}
