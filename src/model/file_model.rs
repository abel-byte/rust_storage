use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct FileHead {
    pub success: bool,
    pub file_hash: String,
    pub file_name: String,
    pub size: usize,
}

impl FileHead {
    pub fn new(success: bool, file_info: FileInfo) -> Self {
        FileHead {
            success,
            file_hash: file_info.file_hash,
            file_name: file_info.file_name,
            size: file_info.size,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileInfo {
    pub file_hash: String,
    pub file_name: String,
    pub size: usize,
    pub content: Vec<u8>,
}

impl FileInfo {
    pub fn new(file_hash: String, file_name: String, size: usize, content: Vec<u8>) -> Self {
        FileInfo {
            file_hash,
            file_name,
            size,
            content,
        }
    }
}
