use anyhow::Result;
use env_logger::Env;
use http_body_util::Full;
use hyper::Error;
use hyper::Response;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::tokio::TokioIo;
use log::error;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::net::TcpListener;

mod metrics;
use metrics::Metrics;

async fn redis_info(m: Arc<Mutex<Metrics>>) -> Result<()> {
    let client = redis::Client::open("redis://127.0.0.1:6379/").unwrap();
    let mut con = client.get_multiplexed_async_connection().await?;

    let info: redis::InfoDict = redis::cmd("INFO").query_async(&mut con).await?;

    // for (k, v) in info.iter() {
    //     info!("{:#?}: {:#?}", k, v);
    // }

    m.lock().unwrap().update(&info);
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let metrics = Arc::new(Mutex::new(Metrics::default()));

    // task for update metrics
    let metrics_clone = metrics.clone();
    tokio::task::spawn(async move {
        loop {
            // TODO: handle error
            let _ = redis_info(metrics_clone.clone()).await;
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        }
    });

    let addr = SocketAddr::from(([0, 0, 0, 0], 8888));

    let listener = TcpListener::bind(addr).await?;

    loop {
        let metrics = metrics.clone();
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        let service = service_fn(move |_req| {
            let mut buffer = String::new();
            metrics.lock().unwrap().encode(&mut buffer);

            async move { Ok::<_, Error>(Response::new(Full::new(Bytes::from(buffer)))) }
        });

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                error!("Error serving connection: {:?}", err);
            }
        });
    }
}
