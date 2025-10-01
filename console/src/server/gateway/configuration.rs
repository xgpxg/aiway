use crate::server::db::models::system_config::{ConfigKey, SystemConfig};
use crate::server::db::Pool;
use protocol::gateway;
use protocol::gateway::{AllowDenyPolicy, Firewall};
use rbs::value;
use std::collections::HashSet;

pub(crate) async fn configuration() -> anyhow::Result<gateway::Configuration> {
    let config = SystemConfig::select_by_map(
        Pool::get()?,
        value! {
            "config_key": ConfigKey::Gateway,
        },
    )
    .await?;
    if config.is_empty() {
        //return Ok(gateway::Configuration::default());
        let mut map = HashSet::new();
        map.insert("127.0.0.1".to_string());
        return Ok(gateway::Configuration {
            firewall: Firewall {
                ip_policy_mode: AllowDenyPolicy::Allow,
                ip_policy: map,//Default::default(),
                referer_policy_mode: Default::default(),
                referer_policy: Default::default(),
                allow_empty_referer: false,
                max_connections: Default::default(),
            },
            ..Default::default()
        });
    }
    let config = config.first().unwrap();
    let config_value = config.config_value.clone().unwrap_or_default();
    if config_value == "" {
        return Ok(gateway::Configuration::default());
    }
    let config: gateway::Configuration = serde_json::from_str(&config_value)?;
    Ok(config)
}
