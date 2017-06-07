use super::KvStore;

use std::path::Path;

use mtbl::{Read, Reader};

impl KvStore for Reader {
    fn get(self: &Self, key: &[u8]) -> Option<Vec<u8>> {
        Read::get(self, key)
    }
}

pub fn new_mtbl(p: &Path) -> Reader {
    Reader::open_from_path(p).unwrap()
}
