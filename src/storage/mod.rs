pub mod rocksdb_storage;
use anyhow::Result;
pub use rocksdb_storage::{rocksdb_conn, DB};

pub trait Storage {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    fn put(&self, key: &str, value: Vec<u8>) -> Result<Option<Vec<u8>>>;
}
