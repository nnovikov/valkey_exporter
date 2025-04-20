use crate::metrics::Metrics;
use crate::metrics::UpdateInfo;
use anyhow::Result;
use anyhow::anyhow;
use futures::stream;
use kameo::Actor;
use kameo::actor::ActorRef;
// use kameo::error::BoxError;
// use kameo::mailbox::unbounded::UnboundedMailbox;
use kameo::error::Infallible;
use kameo::message::StreamMessage;
use kameo::message::{Context, Message};
use log::info;
use tokio::time;
use tokio_stream::StreamExt;

#[derive(Debug, Clone)]
pub struct Poll;

#[derive(Debug)]
pub struct Redis {
    pub client: redis::Client,
    pub inst: String,
    pub poll_interval: u32,
    pub metrics_actor: ActorRef<Metrics>,
}

#[derive(Debug, Default)]
pub struct RedisBuilder {
    inst: Option<String>,
    tls: bool,
    host: Option<String>,
    port: Option<u16>,
    poll_interval: Option<u32>,
    metrics_actor: Option<ActorRef<Metrics>>,
}

impl RedisBuilder {
    pub fn new() -> RedisBuilder {
        RedisBuilder::default()
    }

    pub fn set_inst(&mut self, inst: &str) -> &mut Self {
        self.inst = Some(inst.to_string());
        self
    }

    pub fn host(&mut self, host: &str) -> &mut Self {
        self.host = Some(host.to_string());
        self
    }

    pub fn port(&mut self, port: u16) -> &mut Self {
        self.port = Some(port);
        self
    }

    pub fn poll_interval(&mut self, poll_interval: u32) -> &mut Self {
        self.poll_interval = Some(poll_interval);
        self
    }

    pub fn tls(&mut self, tls: bool) -> &mut Self {
        self.tls = tls;
        self
    }

    pub fn metrics_actor(&mut self, actor_ref: ActorRef<Metrics>) -> &mut Self {
        self.metrics_actor = Some(actor_ref);
        self
    }

    pub fn build(&self) -> Result<Redis> {
        let inst = self.inst.clone().ok_or(anyhow!("inst is none"))?;

        let host = self.host.clone().ok_or(anyhow!("host is none"))?;

        let port = self.port.ok_or(anyhow!("port is none"))?;

        let poll_interval = self.poll_interval.ok_or(anyhow!("poll_interval is none"))?;

        let tls = self.tls;

        let metrics_actor = self
            .metrics_actor
            .clone()
            .ok_or(anyhow!("metrics actor is none"))?;

        let url = if tls {
            format!("rediss://{}:{}", host, port)
        } else {
            format!("redis://{}:{}", host, port)
        };

        Ok(Redis {
            client: redis::Client::open(url).unwrap(),
            inst,
            poll_interval,
            metrics_actor,
        })
    }
}

impl Actor for Redis {
    // type Mailbox = UnboundedMailbox<Self>;
    type Error = Infallible;

    async fn on_start(&mut self, actor_ref: ActorRef<Self>) -> Result<(), Self::Error> {
        println!("actor started");
        let stream = Box::pin(
            stream::repeat(Poll).throttle(time::Duration::from_secs(self.poll_interval as u64)),
        );
        actor_ref.attach_stream(stream, (), ());
        Ok(())
    }
}

impl Message<StreamMessage<Poll, (), ()>> for Redis {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: StreamMessage<Poll, (), ()>,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        info!("{:?} {:?}", msg, self);

        let con = self.client.get_multiplexed_async_connection().await;

        let info = match con {
            Ok(mut con) => redis::cmd("INFO").query_async(&mut con).await.unwrap(),
            Err(e) => {
                info!("connect redis error: {}", e);
                None
            }
        };

        let _ = self
            .metrics_actor
            .tell(UpdateInfo {
                info,
                inst: self.inst.clone(),
            })
            .await;
    }
}
