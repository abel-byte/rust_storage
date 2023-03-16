use crate::model::file_model::FileInfo;
use crate::storage::{rocksdb_conn, Storage, DB};
use anyhow::{Ok, Result};
use tokio::time::Instant;
use tracing::info;

pub async fn save(hash: &str, data: Vec<u8>) -> Result<()> {
    let start = Instant::now();
    let db = DB.get_or_init(rocksdb_conn).await;
    db.put(hash, data)?;
    let duration = start.elapsed();
    info!("save cost {:?}", duration);
    Ok(())
}

pub async fn find(hash: &str) -> Result<FileInfo> {
    let start = Instant::now();
    let db = DB.get_or_init(rocksdb_conn).await;
    let data = db.get(hash)?;
    let file_info: FileInfo = serde_json::from_slice(&data.unwrap())?;
    let duration = start.elapsed();
    info!("find cost {:?}", duration);
    Ok(file_info)
}

pub async fn exists(hash: &str) -> Result<bool> {
    let start = Instant::now();
    let db = DB.get_or_init(rocksdb_conn).await;
    let data = db.exists(hash);
    let duration = start.elapsed();
    info!("exists cost {:?}", duration);
    Ok(data)
}
