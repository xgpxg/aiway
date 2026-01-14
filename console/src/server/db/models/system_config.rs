use crate::server::db::Pool;
use derive_builder::Builder;
use logging::log;
use rbatis::crud;
use rbs::value;
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::sync::RwLock;
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::{Arc, LazyLock};

/// 系统设置
#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
#[builder(default)]
pub struct SystemConfig {
    /// 配置Key
    pub config_key: Option<ConfigKey>,
    /// 配置值
    #[serde(deserialize_with = "crate::server::common::deserialize_to_string")]
    pub config_value: Option<String>,
}

crud!(SystemConfig {});

/// 系统配置项，表中一行一个
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigKey {
    /// 版本号，用于记录当前系统版本，升级时需要
    Version,
    /// 全局路由过滤器配置
    GlobalFilter,
    /// 防火墙配置
    Firewall,
    /// 通知和提醒配置
    Alert,
    /// 最后更新区域调用统计数据的时间，秒级时间戳
    IpRegionLastUpdate,
    /// 最后更新请求状态统计数据时间，秒级时间戳
    RequestStatusCountLastUpdate,
}
impl Display for ConfigKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigKey::Version => write!(f, "version"),
            ConfigKey::GlobalFilter => write!(f, "global-filter"),
            ConfigKey::Firewall => write!(f, "firewall"),
            ConfigKey::Alert => {
                write!(f, "alert")
            }
            ConfigKey::IpRegionLastUpdate => {
                write!(f, "ip-region-last-update")
            }
            ConfigKey::RequestStatusCountLastUpdate => {
                write!(f, "request-status-count-last-update")
            }
        }
    }
}

/// 系统配置缓存
static CACHED_SYSTEM_CONFIG: LazyLock<Arc<RwLock<HashMap<String, SystemConfig>>>> =
    LazyLock::new(Default::default);

impl SystemConfig {
    /// 获取系统配置。
    ///
    /// 优先从内存缓存中获取，如果没有则从数据库中获取并缓存。
    ///
    /// ⚡⚡⚡ 注意：通过`get`获取的地方，在更新数据时，必须调用`upsert`来更新缓存。
    pub async fn get<T: Default + Serialize + for<'a> Deserialize<'a>>(
        config_key: ConfigKey,
    ) -> anyhow::Result<T> {
        // 优先取缓存
        if let Some(cached) = CACHED_SYSTEM_CONFIG
            .read()
            .await
            .get(&config_key.to_string())
            && let Some(value) = &cached.config_value
        {
            log::debug!(
                "hit system config cache, key: {}, value: {}",
                config_key,
                value
            );
            return Ok(serde_json::from_str(value)?);
        }

        // 缓存没有查数据库
        let config =
            SystemConfig::select_by_map(Pool::get()?, value! {"config_key": &config_key}).await?;
        if config.is_empty() {
            log::info!(
                "system config not found, key: {}, set default value to cache",
                config_key
            );
            let default = T::default();
            CACHED_SYSTEM_CONFIG.write().await.insert(
                config_key.to_string(),
                SystemConfig {
                    config_key: Some(config_key.clone()),
                    config_value: Some(serde_json::to_string(&default)?),
                },
            );
            return Ok(default);
        }
        let config = config[0].clone();
        CACHED_SYSTEM_CONFIG
            .write()
            .await
            .insert(config_key.to_string(), config.clone());
        let result = serde_json::from_str(config.config_value.as_ref().unwrap())?;
        Ok(result)
    }

    /// 新增或更新系统配置，并缓存。
    pub async fn upsert<T: Serialize>(config_key: ConfigKey, value: &T) -> anyhow::Result<()> {
        let config =
            SystemConfig::select_by_map(Pool::get().unwrap(), value! {"config_key": &config_key})
                .await?;
        if config.is_empty() {
            SystemConfig::insert(
                Pool::get()?,
                &SystemConfig {
                    config_key: Some(config_key.clone()),
                    config_value: Some(serde_json::to_string(value)?),
                },
            )
            .await?;
        } else {
            SystemConfig::update_by_map(
                Pool::get()?,
                &SystemConfigBuilder::default()
                    .config_value(Some(serde_json::to_string(value)?))
                    .build()?,
                value! {"config_key": &config_key},
            )
            .await?;
        }
        CACHED_SYSTEM_CONFIG.write().await.insert(
            config_key.to_string(),
            SystemConfig {
                config_key: Some(config_key.clone()),
                config_value: Some(serde_json::to_string(value)?),
            },
        );
        Ok(())
    }
}
