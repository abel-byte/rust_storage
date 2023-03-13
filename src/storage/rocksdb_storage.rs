use crate::{config::CONFIG, storage::Storage};
use anyhow::Result;
use rocksdb::DB;
use std::path::Path;
use tokio::sync::OnceCell;

#[derive(Debug)]
pub struct RocksDbStorage(DB);

impl RocksDbStorage {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self(DB::open_default(path).unwrap())
    }
}

impl Storage for RocksDbStorage {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let v = self.0.get(key)?.unwrap();
        Ok(Some(v))
    }

    fn put(&self, key: &str, value: Vec<u8>) -> Result<Option<Vec<u8>>> {
        self.0.put(key, value.clone()).unwrap();
        Ok(Some(value))
    }
}

pub static DB: OnceCell<RocksDbStorage> = OnceCell::const_new();

pub async fn rocksdb_conn() -> RocksDbStorage {
    let db = RocksDbStorage::new(CONFIG.database.url.to_owned());
    tracing::info!("database connected");
    db
}
