use std::sync::Arc;
use std::time::Duration;

use crate::config;
use crate::model::file_model::{FileExists, FileHead, FileInfo, InternalFile, InternalFiles};
use crate::service::file_service;
use crate::util::crypto;
use anyhow::Result;
use poem::{
    handler,
    http::StatusCode,
    web::{Json, Multipart, Path},
    Response,
};
use rand::seq::SliceRandom;
use tokio::task::JoinSet;
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

    let shared_files = Arc::new(InternalFiles::new(internal_files));

    // 除本节点外再选 min_count - 1 个节点
    let mut count = config::CONFIG.cluster.min_count - 1;

    let mut servers = config::CONFIG.cluster.servers.clone();
    servers.remove(&config::CONFIG.server.address);
    let mut new_server: Vec<&String> = servers.iter().collect();

    {
        //  match ret.await {
        //           ^^^^^^ await occurs here, with `mut rng` maybe used later

        let mut rng = rand::thread_rng();
        new_server.shuffle(&mut rng);
        new_server.truncate(count);
    }

    let mut tasks = JoinSet::new();
    for server in new_server {
        tasks.spawn(upload_other(shared_files.clone(), server.clone()));
    }
    while let Some(_resp) = tasks.join_next().await {}

    Ok(Json(files))
}

#[handler]
pub async fn download(Path(file_hash): Path<String>) -> Response {
    // 先查询本节点，有则返回
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

    // 本节点未查询到，从其他节点查询
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
    if internal_server.len() == 0 {
        info!("file not exists");
        return Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body("{{\"exists\": false, \"msg\":\"file not exists\"}}");
    }
    info!("exists in {}", internal_server);

    let resp = download_other(&file_hash, &internal_server);
    let resp = resp.await;

    match resp {
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
pub async fn internal_download(Path(file_hash): Path<String>) -> Response {
    let file = file_service::find(file_hash.as_str()).await;
    match file {
        Ok(f) => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/octet-stream")
            .header(
                "Content-Disposition",
                "attachment;filename=".to_string() + f.file_name.as_str(),
            )
            .body(f.content),
        Err(err) => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(format!("{{\"exists\": false, \"msg\":\"{:?}\"}}", err)),
    }
}

#[handler]
pub async fn internal_exists(Path(file_hash): Path<String>) -> Json<FileExists> {
    let mut file_exists = false;
    if let Ok(f) = file_service::exists(file_hash.as_str()).await {
        file_exists = f;
    }

    Json(FileExists::new(file_exists))
}

async fn upload_other(shared_files: Arc<InternalFiles>, server: String) -> Result<String> {
    let files = shared_files.clone();
    let resp = reqwest::Client::new()
        .post(format!("http://{}/internal/file/upload", server))
        .json(files.as_ref())
        .timeout(Duration::from_secs(30))
        .send()
        .await?;

    Ok(format!(
        "send to {}, status: {}, body: {:?}",
        server,
        resp.status(),
        resp.bytes().await
    ))
}

async fn exists_other(file_hash: &String, server: &String) -> Result<bool> {
    let resp = reqwest::Client::new()
        .get(format!(
            "http://{}/internal/file/{}/exists",
            server, file_hash
        ))
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .unwrap()
        .json::<FileExists>()
        .await
        .unwrap();

    Ok(resp.exists)
}

async fn download_other(file_hash: &String, server: &String) -> Result<Response> {
    let resp = reqwest::Client::new()
        .get(format!("http://{}/internal/file/{}", server, file_hash))
        .timeout(Duration::from_secs(30))
        .send()
        .await?;

    let mut r = poem::Response::default();
    r.set_status(resp.status());
    *r.headers_mut() = resp.headers().clone();
    r.set_body(resp.bytes().await?);

    Ok(r)
}
