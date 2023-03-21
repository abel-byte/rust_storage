use crate::service::file_service;
use internal_files::{
    internal_files_server::InternalFiles, DownloadRequest, DownloadResponse, ExistsRequest,
    ExistsResponse, FileHead, UploadRequest, UploadResponse,
};
use tonic::{Code, Request, Response, Status};
use tracing::info;

pub mod internal_files {
    include!("../proto/internalfiles.rs");
}

#[derive(Debug, Default)]
pub struct InternalFilesService {}

#[tonic::async_trait]
impl InternalFiles for InternalFilesService {
    async fn upload(
        &self,
        request: Request<UploadRequest>,
    ) -> Result<Response<UploadResponse>, Status> {
        info!("receive upload request");
        let req = request.into_inner();

        let mut files: Vec<FileHead> = Vec::new();
        for file in req.files {
            let ret = file_service::save(file.hash.as_str(), file.content.clone()).await;
            let success = match ret {
                Ok(_) => true,
                Err(_) => false,
            };
            files.push(FileHead {
                success: success,
                hash: file.hash.clone(),
                name: file.name.clone(),
                size: file.size,
            });
        }

        Ok(Response::new(UploadResponse { files }))
    }

    async fn exists(
        &self,
        request: Request<ExistsRequest>,
    ) -> Result<Response<ExistsResponse>, Status> {
        info!("receive exists request: {:?}", request);
        let req = request.into_inner();

        let mut exists = false;
        if let Ok(f) = file_service::exists(req.hash.as_str()).await {
            exists = f;
        }

        Ok(Response::new(ExistsResponse { exists }))
    }

    async fn download(
        &self,
        request: Request<DownloadRequest>,
    ) -> Result<Response<DownloadResponse>, Status> {
        info!("receive download request: {:?}", request);
        let req = request.into_inner();

        let file = file_service::find(req.hash.as_str()).await;
        match file {
            Ok(f) => Ok(Response::new(DownloadResponse {
                name: f.file_name,
                content: f.content,
            })),
            Err(err) => Err(Status::new(Code::Internal, format!("{:?}", err))),
        }
    }
}
