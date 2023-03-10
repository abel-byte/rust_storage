use data_encoding::HEXUPPER;
use poem::{
    error::NotFoundError,
    get, handler,
    http::StatusCode,
    listener::TcpListener,
    post,
    web::{Json, Multipart, Path},
    EndpointExt, IntoResponse, Response, Route, Server,
};
use ring::digest::{Context, SHA256};
use rocksdb::{Options, DB};
use serde::{Deserialize, Serialize};

const DB_PATH: &'static str = "./db";

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app = Route::new()
        .at("/", get(index))
        .at("/upload", post(upload))
        .at("/file/:file_hash", get(download))
        .catch_error(|_: NotFoundError| async move {
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("not found")
        });
    let addr = "127.0.0.1:3000";
    println!("listening on {}", addr);
    Server::new(TcpListener::bind(addr)).run(app).await
}

#[handler]
fn index() -> &'static str {
    "Hello, world!"
}

#[handler]
async fn upload(mut multipart: Multipart) -> Json<Vec<FileHead>> {
    let mut files: Vec<FileHead> = Vec::new();
    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = field.file_name().map(ToString::to_string);
        if let Ok(bytes) = field.bytes().await {
            let file_info = FileInfo::new(
                sha256_digest(&bytes),
                file_name.unwrap(),
                bytes.len(),
                bytes,
            );
            let data = serde_json::to_vec(&file_info).unwrap();
            let file_hash = file_info.file_hash.as_str();
            let ret = save(file_hash, data);
            let success = match ret {
                Ok(_) => true,
                Err(_) => false,
            };
            files.push(FileHead::new(success, file_info));
        }
    }
    Json(files)
}

#[handler]
async fn download(Path(file_hash): Path<String>) -> impl IntoResponse {
    let mut file_name = String::from("");
    let mut data = Vec::new();
    if let Ok(f) = find(file_hash.as_str()) {
        file_name = f.file_name;
        data = f.content;
    }

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/octet-stream")
        .header(
            "Content-Disposition",
            "attachment;filename=".to_string() + file_name.as_str(),
        )
        .body(data)
}

#[derive(Serialize, Deserialize, Debug)]
struct FileHead {
    success: bool,
    file_hash: String,
    file_name: String,
    size: usize,
}

impl FileHead {
    fn new(success: bool, file_info: FileInfo) -> Self {
        FileHead {
            success,
            file_hash: file_info.file_hash,
            file_name: file_info.file_name,
            size: file_info.size,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct FileInfo {
    file_hash: String,
    file_name: String,
    size: usize,
    content: Vec<u8>,
}

impl FileInfo {
    fn new(file_hash: String, file_name: String, size: usize, content: Vec<u8>) -> Self {
        FileInfo {
            file_hash,
            file_name,
            size,
            content,
        }
    }
}

fn save(hash: &str, data: Vec<u8>) -> anyhow::Result<()> {
    let mut opts = Options::default();
    opts.create_if_missing(true);
    let db = DB::open(&opts, DB_PATH)?;

    db.put(hash.as_bytes(), data)?;
    Ok(())
}

fn find(hash: &str) -> anyhow::Result<FileInfo> {
    let mut opts = Options::default();
    opts.create_if_missing(true);
    let db = DB::open(&opts, DB_PATH)?;

    let data = db.get(hash.as_bytes())?;

    let file_info: FileInfo = serde_json::from_slice(&data.unwrap())?;
    Ok(file_info)
}

fn sha256_digest(buffer: &Vec<u8>) -> String {
    let mut context = Context::new(&SHA256);
    if buffer.len() == 0 {
        return String::from("");
    }
    context.update(&buffer);
    let digest = context.finish();
    let signature = HEXUPPER.encode(digest.as_ref());
    signature
}
