use crate::model::file_model::FileInfo;
use crate::storage::{rocksdb_conn, Storage, DB};
use anyhow::{Ok, Result};

pub async fn save(hash: &str, data: Vec<u8>) -> Result<()> {
    let db = DB.get_or_init(rocksdb_conn).await;
    db.put(hash, data)?;
    Ok(())
}

pub async fn find(hash: &str) -> Result<FileInfo> {
    let db = DB.get_or_init(rocksdb_conn).await;
    let data = db.get(hash)?;
    let file_info: FileInfo = serde_json::from_slice(&data.unwrap())?;
    Ok(file_info)
}

pub async fn exists(hash: &str) -> Result<bool> {
    let db = DB.get_or_init(rocksdb_conn).await;
    let data = db.exists(hash);
    Ok(data)
}
