use crate::server::db::Pool;
use crate::server::db::models::system_config::{ConfigKey, SystemConfig};
use protocol::gateway::{AllowDenyPolicy, Firewall};
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
        return Ok(Firewall {
            ip_policy_mode: AllowDenyPolicy::Disable,
            ip_policy: Default::default(),
            trust_ips: Default::default(),
            referer_policy_mode: Default::default(),
            referer_policy: Default::default(),
            allow_empty_referer: false,
            max_connections: Default::default(),
        });
    }
    let config = config.first().unwrap();
    let config_value = config.config_value.clone().unwrap_or_default();
    if config_value.is_empty() {
        return Ok(Firewall::default());
    }
    let config: Firewall = serde_json::from_str(&config_value)?;
    Ok(config)
}
