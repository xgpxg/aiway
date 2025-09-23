//! # 发布订阅
//! 基于NATS，消息内容统一使用JSON格式。
//!
//! 使用方式：
//! ```rust
//! // 初始化
//! pubsub::init("127.0.0.1:4222").await.unwrap();
//!
//! // 发布消息
//! pubsub::publish("topic","xxx").await.unwrap();
//!
//! // 订阅消息
//! let subscriber = pubsub::subscribe("topic").await.unwrap();
//!
//! while let Some(message) = subscriber.next().await {
//!     println!("Received message {:?}", message);
//! }
//! ```
//!
//! # TODO 消息确认机制待研究
//!
use async_nats::Client;
use async_nats::subject::ToSubject;
use logging::log;
use serde::Serialize;
use std::fmt::{Debug, Display, Formatter};
use std::sync::OnceLock;

type Publisher = async_nats::jetstream::Context;
type Subscriber = async_nats::Subscriber;

pub struct PubSub {
    client: Client,
    publisher: Publisher,
}

#[derive(Debug)]
pub enum Err {
    ConnectError(async_nats::ConnectError),
    SubscribeError(async_nats::SubscribeError),
    PublishError(async_nats::jetstream::context::PublishError),
    SerializeError(serde_json::Error),
    NotInit(String),
}
impl Display for Err {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
impl std::error::Error for Err {}

impl PubSub {
    pub async fn connect<P: AsRef<str>>(addr: P) -> Result<Self, Err> {
        let client = async_nats::connect(addr.as_ref())
            .await
            .map_err(|e| Err::ConnectError(e))?;
        Ok(Self {
            client: client.clone(),
            publisher: async_nats::jetstream::new(client),
        })
    }
    pub async fn subscribe<T: ToSubject>(&self, topic: T) -> Result<Subscriber, Err> {
        let subscriber = self
            .client
            .subscribe(topic)
            .await
            .map_err(|e| Err::SubscribeError(e))?;
        Ok(subscriber)
    }

    pub async fn publish<T: ToSubject, M: Serialize>(
        &self,
        topic: T,
        message: M,
    ) -> Result<(), Err> {
        let message = serde_json::to_vec(&message).map_err(|e| Err::SerializeError(e))?;
        self.publisher
            .publish(topic, message.into())
            .await
            .map_err(|e| Err::PublishError(e))?;
        Ok(())
    }
}

pub static PUB_SUB: OnceLock<PubSub> = OnceLock::new();

pub async fn init(addr: &str) -> Result<(), Err> {
    log::info!("init pub-sub client");

    let ps = PubSub::connect(addr).await?;

    PUB_SUB.get_or_init(|| ps);

    Ok(())
}

pub async fn publish(topic: impl ToSubject, message: impl Serialize) -> Result<(), Err> {
    match PUB_SUB.get() {
        Some(ps) => ps.publish(topic, message).await,
        None => Err(Err::NotInit("pub-sub not initialized".to_string())),
    }
}
pub async fn subscribe(topic: impl ToSubject) -> Result<Subscriber, Err> {
    match PUB_SUB.get() {
        Some(ps) => ps.subscribe(topic).await,
        None => Err(Err::NotInit("pub-sub not initialized".to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;
    use serde::{Deserialize, Serialize};

    #[tokio::test]
    async fn test_pub_sub() -> Result<(), Box<dyn std::error::Error>> {
        let ps = PubSub::connect("127.0.0.1:4222").await?;
        let mut s = ps.subscribe("gateway.log").await?;

        #[derive(Debug, Serialize, Deserialize)]
        struct Foo {
            name: String,
        }

        /*tokio::spawn(async move {
            loop {
                ps.publish(
                    "foo",
                    Foo {
                        name: "张三".to_string(),
                    },
                )
                .await
                .unwrap();
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        });*/

        while let Some(message) = s.next().await {
            println!("Received message {:?}", message);
        }

        Ok(())
    }
}
