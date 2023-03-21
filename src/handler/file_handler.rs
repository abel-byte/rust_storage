use crate::config;
use crate::handler::file_grpc_handler::internal_files::{
    internal_files_client::InternalFilesClient, DownloadRequest, ExistsRequest, InternalFile,
    UploadRequest,
};
use crate::model::file_model::{FileHead, FileInfo};
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
            internal_files.push(InternalFile {
                hash: file_hash,
                name: file_name,
                content: data,
                size: file_info.size as i64,
            });
        }
    }

    // 除本节点外再选 min_count - 1 个节点
    let mut count = config::CONFIG.cluster.min_count - 1;

    let mut servers = config::CONFIG.cluster.servers.clone();
    let mut local = String::from("http://") + &config::CONFIG.server.grpc_address;
    servers.remove(&local);
    let mut new_servers: Vec<&String> = servers.iter().collect();

    {
        //  match ret.await {
        //           ^^^^^^ await occurs here, with `mut rng` maybe used later

        let mut rng = rand::thread_rng();
        new_servers.shuffle(&mut rng);
        new_servers.truncate(count);
    }

    let mut tasks = JoinSet::new();
    for server in new_servers {
        tasks.spawn(upload_other(
            UploadRequest {
                files: internal_files.clone(),
            },
            server.clone(),
        ));
    }
    while let Some(resp) = tasks.join_next().await {
        info!("{:?}", resp);
    }

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
    let mut count = servers.len() - 1;
    let mut local = String::from("http://") + &config::CONFIG.server.grpc_address;
    servers.remove(&local);
    let mut new_servers: Vec<&String> = servers.iter().collect();

    {
        //  match ret.await {
        //           ^^^^^^ await occurs here, with `mut rng` maybe used later

        let mut rng = rand::thread_rng();
        new_servers.shuffle(&mut rng);
        new_servers.truncate(count);
    }

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

async fn upload_other(upload_request: UploadRequest, server: String) -> Result<String> {
    let mut client = InternalFilesClient::connect(server.clone()).await?;
    let request = tonic::Request::new(upload_request);

    let result = client.upload(request).await;
    match result {
        Ok(response) => Ok(format!(
            "send to {}, response: {:?}",
            server,
            response.into_inner().files
        )),
        Err(err) => Ok(format!("send to {} failed, response: {:?}", server, err)),
    }
}

async fn exists_other(file_hash: &String, server: &String) -> Result<bool> {
    let exists_request = ExistsRequest {
        hash: file_hash.to_string(),
    };

    let mut client = InternalFilesClient::connect(server.clone()).await?;
    let request = tonic::Request::new(exists_request);

    let result = client.exists(request).await;
    match result {
        Ok(response) => Ok(response.into_inner().exists),
        Err(_) => Ok(false),
    }
}

async fn download_other(file_hash: &String, server: &String) -> Result<Response> {
    let download_request = DownloadRequest {
        hash: file_hash.to_string(),
    };

    let mut client = InternalFilesClient::connect(server.clone()).await?;
    let request = tonic::Request::new(download_request);

    let result = client.download(request).await;
    match result {
        Ok(response) => {
            let res = response.into_inner();
            let r = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/octet-stream")
                .header(
                    "Content-Disposition",
                    "attachment;filename=".to_string() + res.name.as_str(),
                )
                .body(res.content);
            Ok(r)
        }
        Err(err) => {
            let r = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(format!("{{\"exists\": false, \"msg\":\"{:?}\"}}", err));
            Ok(r)
        }
    }
}
