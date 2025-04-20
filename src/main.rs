use anyhow::Result;
use clap::Parser;
use env_logger::Env;
use futures::future::BoxFuture;
use http_body_util::Full;
use hyper::Request;
use hyper::Response;
use hyper::body::Bytes;
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::tokio::TokioIo;
use kameo::actor::ActorRef;
use log::error;
use log::info;
use std::io;
use std::path::PathBuf;
use tokio::net::TcpListener;

mod config;

mod metrics;
use metrics::Metrics;

mod redis_actor;
use redis_actor::RedisBuilder;

#[derive(Parser, Debug)]
struct Opts {
    #[arg(short, long)]
    config: PathBuf,
}

fn http_handler(
    actor_ref: ActorRef<Metrics>,
) -> impl Fn(Request<Incoming>) -> BoxFuture<'static, io::Result<Response<Full<Bytes>>>> {
    move |_req| {
        let actor_ref = actor_ref.clone();
        Box::pin(async move {
            let buf = actor_ref.ask(metrics::Encode {}).await.unwrap();
            // let mut buf = String::new();
            let body = Full::new(Bytes::from(buf));
            let resp = Response::builder()
                .header(
                    hyper::header::CONTENT_TYPE,
                    "application/openmetrics-text; version=1.0.0; charset=utf-8",
                )
                .body(body)
                .unwrap();
            Ok(resp)
        })
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let opts = Opts::parse();

    let config = config::parse_config(&opts.config)?;

    let mut level = "info";

    if config.options.debug {
        level = "debug";
    }

    env_logger::Builder::from_env(Env::default().default_filter_or(level)).init();

    info!("{:#?}", config);

    let metrics = Metrics::with_prefix(&config.options.metric_prefix);

    let metrics_actor = kameo::spawn(metrics);

    // start redis actors
    for redis in config.redis_list {
        let redis_actor = RedisBuilder::new()
            .set_inst(&redis.inst)
            .host(&redis.host)
            .port(redis.port)
            .poll_interval(redis.poll_interval)
            .metrics_actor(metrics_actor.clone())
            .tls(redis.tls)
            .build()?;
        let _ = kameo::spawn(redis_actor);
    }

    let listen = config.options.listen;
    let listener = TcpListener::bind(listen).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        let service = service_fn(http_handler(metrics_actor.clone()));

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                error!("Error serving connection: {:?}", err);
            }
        });
    }
}
