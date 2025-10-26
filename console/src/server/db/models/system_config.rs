use derive_builder::Builder;
use rbatis::crud;
use rocket::serde::{Deserialize, Serialize};
use std::fmt::Display;

/// 系统设置
#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
#[builder(default)]
pub struct SystemConfig {
    pub id: Option<i64>,
    /// 配置Key
    pub config_key: Option<ConfigKey>,
    /// 配置值
    #[serde(deserialize_with = "crate::server::common::deserialize_to_string")]
    pub config_value: Option<String>,
}

crud!(SystemConfig {});

/// 系统配置项，一行一个
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigKey {
    /// 版本号
    Version,
    /// 网关配置
    Gateway,
    /// 防火墙配置
    Firewall,
}
impl Display for ConfigKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigKey::Version => write!(f, "version"),
            ConfigKey::Gateway => write!(f, "gateway"),
            ConfigKey::Firewall => write!(f, "firewall"),
        }
    }
}
