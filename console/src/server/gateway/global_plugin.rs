use crate::server::db::Pool;
use crate::server::db::models::system_config::{ConfigKey, SystemConfig};
use aiway_protocol::gateway;
use rbs::value;
pub(crate) async fn plugins() -> anyhow::Result<gateway::GlobalPlugin> {
    let config = SystemConfig::select_by_map(
        Pool::get()?,
        value! {
            "config_key": ConfigKey::GlobalPlugin,
        },
    )
    .await?;
    if config.is_empty() {
        //return Ok(gateway::Configuration::default());
        // let mut map = HashSet::new();
        // map.insert("127.0.0.1".to_string());
        return Ok(gateway::GlobalPlugin {
            ..Default::default()
        });
    }
    let config = config.first().unwrap();
    let config_value = config.config_value.clone().unwrap_or_default();
    if config_value.is_empty() {
        return Ok(gateway::GlobalPlugin::default());
    }
    let config: gateway::GlobalPlugin = serde_json::from_str(&config_value)?;
    Ok(config)
}
