use crate::server::db::models::system_config::{ConfigKey, SystemConfig};
use crate::server::firewall::request::FirewallUpdateReq;
use aiway_protocol::gateway::Firewall;

pub async fn update(req: FirewallUpdateReq) -> anyhow::Result<()> {
    SystemConfig::upsert(ConfigKey::Firewall, &req.inner).await
}

pub async fn detail() -> anyhow::Result<Firewall> {
    SystemConfig::get(ConfigKey::Firewall).await
}
