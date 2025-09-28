use crate::server::db::Pool;
use crate::server::db::models::system_config::{ConfigKey, SystemConfig};
use protocol::gateway;
use rbs::value;

pub(crate) async fn configuration() -> anyhow::Result<gateway::Configuration> {
    let config = SystemConfig::select_by_map(
        Pool::get()?,
        value! {
            "config_key": ConfigKey::Gateway,
        },
    )
    .await?;
    if config.is_empty() {
        return Ok(gateway::Configuration::default());
    }
    let config = config.first().unwrap();
    let config_value = config.config_value.clone().unwrap_or_default();
    if config_value == "" {
        return Ok(gateway::Configuration::default());
    }
    let config: gateway::Configuration = serde_json::from_str(&config_value)?;
    Ok(config)
}
