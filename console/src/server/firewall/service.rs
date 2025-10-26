use crate::server::db::Pool;
use crate::server::db::models::system_config::{ConfigKey, SystemConfig, SystemConfigBuilder};
use crate::server::firewall::request::FirewallUpdateReq;
use protocol::gateway::Firewall;
use rbs::value;

pub async fn update(req: FirewallUpdateReq) -> anyhow::Result<()> {
    // 查询旧的
    let config = SystemConfig::select_by_map(
        Pool::get()?,
        value! {
            "config_key": ConfigKey::Firewall,
        },
    )
    .await?;

    let system_config = SystemConfigBuilder::default()
        .config_key(Some(ConfigKey::Firewall))
        .config_value(Some(serde_json::to_string(&req.inner)?))
        .build()?;

    if config.is_empty() {
        SystemConfig::insert(Pool::get()?, &system_config).await?;
    } else {
        SystemConfig::update_by_map(
            Pool::get()?,
            &system_config,
            value! {
                "config_key": ConfigKey::Firewall,
            },
        )
        .await?;
    }

    Ok(())
}

pub async fn detail() -> anyhow::Result<Firewall> {
    // 查询旧的
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

    let firewall = serde_json::from_str::<Firewall>(&config[0].clone().config_value.unwrap())?;

    Ok(firewall)
}
