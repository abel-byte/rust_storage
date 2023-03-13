use crate::model::file_model::{FileHead, FileInfo};
use crate::service::file_service;
use crate::util::crypto;
use poem::{
    handler,
    http::StatusCode,
    web::{Json, Multipart, Path},
    IntoResponse, Response,
};

#[handler]
pub async fn upload(mut multipart: Multipart) -> Json<Vec<FileHead>> {
    let mut files: Vec<FileHead> = Vec::new();
    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = field.file_name().map(ToString::to_string);
        if let Ok(bytes) = field.bytes().await {
            let file_info = FileInfo::new(
                crypto::sha256_digest(&bytes),
                file_name.unwrap(),
                bytes.len(),
                bytes,
            );
            let data = serde_json::to_vec(&file_info).unwrap();
            let file_hash = file_info.file_hash.as_str();
            let ret = file_service::save(file_hash, data).await;
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
pub async fn download(Path(file_hash): Path<String>) -> impl IntoResponse {
    // 先查询本机，有则返回
    if let Ok(f) = file_service::find(file_hash.as_str()).await {
        return Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/octet-stream")
            .header(
                "Content-Disposition",
                "attachment;filename=".to_string() + f.file_name.as_str(),
            )
            .body(f.content);
    }

    return Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/text")
        .body("not found");
}

#[handler]
pub async fn internal_upload(mut multipart: Multipart) -> Json<Vec<FileHead>> {
    let mut files: Vec<FileHead> = Vec::new();
    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = field.file_name().map(ToString::to_string);
        if let Ok(bytes) = field.bytes().await {
            let file_info = FileInfo::new(
                crypto::sha256_digest(&bytes),
                file_name.unwrap(),
                bytes.len(),
                bytes,
            );
            let data = serde_json::to_vec(&file_info).unwrap();
            let file_hash = file_info.file_hash.as_str();
            let ret = file_service::save(file_hash, data).await;
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
pub async fn internal_download(Path(file_hash): Path<String>) -> impl IntoResponse {
    let mut file_name = String::from("");
    let mut data = Vec::new();
    if let Ok(f) = file_service::find(file_hash.as_str()).await {
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
