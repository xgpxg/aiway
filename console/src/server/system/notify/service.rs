use crate::server::db::models::system_config::{ConfigKey, SystemConfig};
use crate::server::system::notify::request::NotifyConfigUpdateReq;
use aiway_protocol::gateway::alert::AlertConfig;

pub async fn update(req: NotifyConfigUpdateReq) -> anyhow::Result<()> {
    SystemConfig::upsert(ConfigKey::Alert, &req.inner).await?;
    Ok(())
}

pub(crate) async fn get() -> anyhow::Result<AlertConfig> {
    SystemConfig::get(ConfigKey::Alert).await
}
