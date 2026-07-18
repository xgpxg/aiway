use crate::server::db::models::system_config::{ConfigKey, SystemConfig};
use crate::server::system::agent_entry::request::AgentEntryConfig;

pub async fn update(req: AgentEntryConfig) -> anyhow::Result<()> {
    SystemConfig::upsert(ConfigKey::AgentEntryUrl, &req.entry_url).await?;
    Ok(())
}

pub async fn get() -> anyhow::Result<AgentEntryConfig> {
    let entry_url: String = SystemConfig::get(ConfigKey::AgentEntryUrl).await?;
    Ok(AgentEntryConfig { entry_url })
}
