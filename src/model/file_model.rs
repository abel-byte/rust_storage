use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct FileHead {
    pub success: bool,
    pub file_hash: String,
    pub file_name: String,
    pub size: usize,
}

impl FileHead {
    pub fn new(success: bool, file_hash: String, file_name: String, size: usize) -> Self {
        FileHead {
            success,
            file_hash,
            file_name,
            size,
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

#[derive(Debug, Deserialize, Serialize)]
pub struct InternalFiles {
    pub files: Vec<InternalFile>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InternalFile {
    pub file_hash: String,
    pub file_name: String,
    pub file_content: Vec<u8>,
    pub size: usize,
}

impl InternalFiles {
    pub fn new(files: Vec<InternalFile>) -> Self {
        InternalFiles { files }
    }
}

impl InternalFile {
    pub fn new(file_hash: String, file_name: String, file_content: Vec<u8>, size: usize) -> Self {
        InternalFile {
            file_hash,
            file_name,
            file_content,
            size,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FileExists {
    pub exists: bool,
}

impl FileExists {
    pub fn new(exists: bool) -> Self {
        FileExists { exists }
    }
}
