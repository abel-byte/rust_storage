use anyhow::Result;
use poem::{
    error::NotFoundError, get, http::StatusCode, listener::TcpListener, post, EndpointExt,
    Response, Route, Server,
};
use rust_storage::handler::common_handler::index;
use rust_storage::handler::file_handler::{
    download, internal_download, internal_exists, internal_upload, upload,
};
use rust_storage::{config, middleware_fn::log};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    tracing_subscriber::fmt::init();

    let app = Route::new()
        .at("/", get(index))
        .at("/file/upload", post(upload))
        .at("/file/:file_hash", get(download))
        .at("/internal/file/upload", post(internal_upload))
        .at("/internal/file/:file_hash", get(internal_download))
        .at("/internal/file/:file_hash/exists", get(internal_exists))
        .with(log::Log)
        .catch_error(|_: NotFoundError| async move {
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("not found")
        });
    let addr = config::CONFIG.server.address.to_owned();
    info!("listening on {}", addr);
    Server::new(TcpListener::bind(addr)).run(app).await
}
