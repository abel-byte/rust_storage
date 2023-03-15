use crate::config;
use crate::model::file_model::{FileExists, FileHead, FileInfo, InternalFile, InternalFiles};
use crate::service::file_service;
use crate::util::crypto;
use poem::{
    error::BadRequest,
    handler,
    http::StatusCode,
    web::{Json, Multipart, Path},
    IntoResponse, Response, Result,
};
use rand::seq::SliceRandom;
use tracing::info;

#[handler]
pub async fn upload(mut multipart: Multipart) -> Result<Json<Vec<FileHead>>> {
    let mut files: Vec<FileHead> = Vec::new();
    let mut internal_files: Vec<InternalFile> = Vec::new();
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
            let file_hash = file_info.file_hash.clone();
            let file_name = file_info.file_name;
            let ret = file_service::save(file_hash.as_str(), data.clone()).await;
            let success = match ret {
                Ok(_) => true,
                Err(_) => false,
            };
            files.push(FileHead::new(
                success,
                file_hash.clone(),
                file_name.clone(),
                file_info.size,
            ));
            internal_files.push(InternalFile::new(
                file_hash,
                file_name,
                data,
                file_info.size,
            ));
        }
    }

    let internal_files = InternalFiles::new(internal_files);

    // 除本机外再选 min_count - 1 个节点
    let mut servers = config::CONFIG.cluster.servers.clone();
    servers.remove(&config::CONFIG.server.address);
    let mut count = config::CONFIG.cluster.min_count - 1;
    let mut new_servers: Vec<&String> = servers.iter().collect();

    {
        //  match ret.await {
        //           ^^^^^^ await occurs here, with `mut rng` maybe used later

        let mut rng = rand::thread_rng();
        new_servers.shuffle(&mut rng);
        new_servers.truncate(count);
    }

    for server in new_servers {
        let ret = upload_other(&internal_files, server);
        match ret.await {
            Ok(resp) => info!("sending to {}, result: {:?}", server, resp),
            Err(err) => info!("sending to {}, result: {:?}", server, err),
        }
    }

    Ok(Json(files))
}

#[handler]
pub async fn download(Path(file_hash): Path<String>) -> poem::Response {
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

    // 本机未查询到，至少 Count(servers) - min_count - 1 个节点
    let mut servers = config::CONFIG.cluster.servers.clone();
    servers.remove(&config::CONFIG.server.address);
    let mut new_servers: Vec<&String> = servers.iter().collect();

    let mut internal_server = String::from("");
    for server in new_servers {
        let ret = exists_other(&file_hash, server);
        match ret.await {
            Ok(resp) => {
                if resp {
                    internal_server = server.clone();
                    break;
                }
            }
            Err(err) => info!("sending to {}, result: {:?}", server, err),
        }
    }
    info!("exists in {}", internal_server);

    let resp = download_other(&file_hash, &internal_server);
    match resp.await {
        Ok(resp) => resp,
        Err(err) => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(format!("{{\"exists\": false, \"msg\":\"{}\"}}", err)),
    }
}

#[handler]
pub async fn internal_upload(req: Json<InternalFiles>) -> Json<Vec<FileHead>> {
    let mut files: Vec<FileHead> = Vec::new();
    let req_files = &req.files;
    for file in req_files {
        let ret = file_service::save(file.file_hash.as_str(), file.file_content.clone()).await;
        let success = match ret {
            Ok(_) => true,
            Err(_) => false,
        };
        files.push(FileHead::new(
            success,
            file.file_hash.clone(),
            file.file_name.clone(),
            file.size,
        ));
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

#[handler]
pub async fn internal_exists(Path(file_hash): Path<String>) -> Json<FileExists> {
    let mut file_exists = false;
    if let Ok(f) = file_service::exists(file_hash.as_str()).await {
        file_exists = f;
    }

    Json(FileExists::new(file_exists))
}

async fn upload_other(files: &InternalFiles, server: &String) -> Result<poem::Response> {
    let resp = reqwest::Client::new()
        .post(format!("http://{}/internal/file/upload", server))
        .json(&files)
        .send()
        .await
        .map_err(BadRequest)?;

    let mut r = poem::Response::default();
    r.set_status(resp.status());
    *r.headers_mut() = resp.headers().clone();
    r.set_body(resp.bytes().await.map_err(BadRequest)?);
    Ok(r)
}

async fn exists_other(file_hash: &String, server: &String) -> Result<bool> {
    let resp = reqwest::Client::new()
        .get(format!(
            "http://{}/internal/file/{}/exists",
            server, file_hash
        ))
        .send()
        .await
        .unwrap()
        .json::<FileExists>()
        .await
        .unwrap();

    Ok(resp.exists)
}

async fn download_other(file_hash: &String, server: &String) -> Result<poem::Response> {
    let resp = reqwest::Client::new()
        .get(format!("http://{}/internal/file/{}", server, file_hash))
        .send()
        .await
        .map_err(BadRequest)?;

    let mut r = poem::Response::default();
    r.set_status(resp.status());
    *r.headers_mut() = resp.headers().clone();
    r.set_body(resp.bytes().await.map_err(BadRequest)?);
    Ok(r)
}
