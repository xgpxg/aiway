use crate::server::db::Pool;
use crate::server::db::models::system_config::{ConfigKey, SystemConfig};
use aiway_protocol::gateway::Firewall;
use rbs::value;

pub(crate) async fn configuration() -> anyhow::Result<Firewall> {
    let config = SystemConfig::select_by_map(
        Pool::get()?,
        value! {
            "config_key": ConfigKey::Firewall,
        },
    )
    .await?;
    if config.is_empty() {
        return Ok(Firewall::default());
    }
    let config = config.first().unwrap();
    let config_value = config.config_value.clone().unwrap_or_default();
    if config_value.is_empty() {
        return Ok(Firewall::default());
    }
    let config: Firewall = serde_json::from_str(&config_value)?;
    Ok(config)
}
