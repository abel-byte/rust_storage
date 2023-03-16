use poem::{async_trait, Endpoint, IntoResponse, Middleware, Request, Response, Result};
use tokio::time::Instant;
use tracing::info;

pub struct Log;

impl<E: Endpoint> Middleware<E> for Log {
    type Output = LogImpl<E>;

    fn transform(&self, ep: E) -> Self::Output {
        LogImpl(ep)
    }
}

pub struct LogImpl<E>(E);

#[async_trait]
impl<E: Endpoint> Endpoint for LogImpl<E> {
    type Output = Response;

    async fn call(&self, req: Request) -> Result<Self::Output> {
        let start = Instant::now();
        let path = req.uri().path().to_string();
        info!("request {}", path);
        let res = self.0.call(req).await;
        let duration = start.elapsed();

        match res {
            Ok(resp) => {
                let resp = resp.into_response();
                info!(
                    "request {} cost {:?}, response: {}",
                    path,
                    duration,
                    resp.status()
                );
                Ok(resp)
            }
            Err(err) => {
                info!("request {} cost {:?}, error: {}", path, duration, err);
                Err(err)
            }
        }
    }
}
