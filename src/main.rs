use anyhow::{Ok, Result};
use poem::{
    error::NotFoundError, get, http::StatusCode, listener::TcpListener, post, EndpointExt,
    Response, Route, Server,
};
use rust_storage::handler::common_handler::index;
use rust_storage::handler::file_grpc_handler::{
    internal_files::internal_files_server::InternalFilesServer, InternalFilesService,
};
use rust_storage::handler::file_handler::{download, upload};
use rust_storage::{config, middleware_fn::log};
use tokio::join;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let http_server = http_server();
    let grpc_server = grpc_server();
    let _ = join!(http_server, grpc_server);

    Ok(())
}

async fn http_server() -> Result<()> {
    let app = Route::new()
        .at("/", get(index))
        .at("/file/upload", post(upload))
        .at("/file/:file_hash", get(download))
        .with(log::Log)
        .catch_error(|_: NotFoundError| async move {
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("not found")
        });
    let addr = config::CONFIG.server.address.to_owned();
    info!("listening on {}", addr);
    Server::new(TcpListener::bind(addr)).run(app).await?;
    Ok(())
}

async fn grpc_server() -> Result<()> {
    let addr = config::CONFIG.server.grpc_address.to_owned().parse()?;
    info!("grpc listening on {}", addr);
    let service = InternalFilesService::default();
    tonic::transport::Server::builder()
        .add_service(InternalFilesServer::new(service))
        .serve(addr)
        .await?;
    Ok(())
}
