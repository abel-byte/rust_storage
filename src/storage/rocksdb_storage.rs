use crate::{config::CONFIG, storage::Storage};
use anyhow::{anyhow, Ok, Result};
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
        let v = self.0.get(key)?;
        match v {
            Some(val) => Ok(Some(val)),
            None => Err(anyhow!("not found")),
        }
    }

    fn exists(&self, key: &str) -> bool {
        self.0.key_may_exist(key)
    }

    fn put(&self, key: &str, value: Vec<u8>) -> Result<()> {
        if !self.exists(key) {
            self.0.put(key, value.clone())?;
        }
        Ok(())
    }
}

pub static DB: OnceCell<RocksDbStorage> = OnceCell::const_new();

pub async fn rocksdb_conn() -> RocksDbStorage {
    let db = RocksDbStorage::new(CONFIG.database.url.to_owned());
    tracing::info!("database connected");
    db
}
