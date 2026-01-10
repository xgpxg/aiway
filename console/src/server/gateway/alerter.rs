use crate::server::db::Pool;
use crate::server::db::models::message::{Message, MessageBuilder, MessageReadStatus};
use crate::server::db::models::system_config::{ConfigKey, SystemConfig};
use alert::pusher::Pusher;
use common::id;
use logging::log;
use aiway_protocol::gateway::alert::{AlertConfig, AlertMessage};
use rbatis::rbdc::DateTime;
use rocket::tokio;
use std::str::FromStr;

/// 接收并推送告警消息
pub(crate) async fn alert(req: AlertMessage) -> anyhow::Result<()> {
    // 获取推送配置并推送消息
    let config = SystemConfig::get::<AlertConfig>(ConfigKey::Alert).await?;
    Pusher::push(config.into(), req.clone());

    // 异步入库
    tokio::spawn(async move {
        let message = MessageBuilder::default()
            .id(Some(id::next()))
            .level(Some(req.level))
            .title(Some(req.title))
            .content(Some(req.content))
            // 创建时间取告警发生时间
            .create_time(Some(DateTime::from_str(&req.time)?))
            .read_status(Some(MessageReadStatus::Unread))
            .build()?;
        match Message::insert(Pool::get()?, &message).await {
            Ok(_) => {}
            Err(e) => {
                log::error!("insert message error: {}", e);
            }
        }

        Ok::<(), anyhow::Error>(())
    });

    Ok(())
}
