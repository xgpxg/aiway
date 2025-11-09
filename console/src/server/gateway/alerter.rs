use crate::server::db::models::system_config::{ConfigKey, SystemConfig};
use alert::pusher::Pusher;
use protocol::gateway::alert::{AlertConfig, AlertMessage};

/// 接收并推送告警消息
pub(crate) async fn alert(req: AlertMessage) -> anyhow::Result<()> {
    let config = SystemConfig::get::<AlertConfig>(ConfigKey::Alert).await?;
    Pusher::push(config.into(), req);
    Ok(())
}
