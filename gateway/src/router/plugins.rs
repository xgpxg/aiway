//! # 插件
//! 负责从控制台加载所有已启用的插件并缓存。
//!
//! 实现流程：
//! - 初始化时，尝试从控制台的`GET /api/v1/gateway/plugins`端点获取插件列表。
//! - 如果控制台无法连接，则退出，禁止启动。
//! - 缓存插件列表到内存以及本地。
//! - 启动定时任务，每5秒从控制台拉取插件列表，校验hash值，如果不一致则更新本地插件列表。
//!
//! 注意：该组件会保存所有有效的插件实例，如果需要调用插件，必须通过插件名称获取实例后执行。
//!

use crate::Args;
use crate::router::client::INNER_HTTP_CLIENT;
use anyhow::bail;
use clap::Parser;
use dashmap::DashMap;
use plugin::{AsyncTryInto, NetworkPlugin, Plugin};
use protocol::gateway::plugin::ConfiguredPlugin;
use protocol::gateway::{HttpContext, Plugin as PluginConfig};
use std::process::exit;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::RwLock;

pub struct PluginFactory {
    pub plugins: DashMap<String, (PluginConfig, Box<dyn Plugin>)>,
    pub hash: Arc<RwLock<String>>,
}

pub static PLUGINS: OnceLock<PluginFactory> = OnceLock::new();

impl PluginFactory {
    pub async fn init() {
        if let Err(e) = Self::load().await {
            log::error!("{}", e);
            exit(1)
        }
    }

    pub async fn load() -> anyhow::Result<()> {
        let list = Self::fetch_plugins().await?;

        log::info!("loaded {} plugins", list.len());

        let hash = md5::compute(serde_json::to_string(&list)?);
        let hash = format!("{:x}", hash);

        let plugins = Self::process_plugins(list).await?;

        PLUGINS.get_or_init(|| Self {
            plugins,
            hash: Arc::new(RwLock::new(hash)),
        });

        Self::watch();

        Ok(())
    }

    async fn process_plugins(
        list: Vec<PluginConfig>,
    ) -> anyhow::Result<DashMap<String, (PluginConfig, Box<dyn Plugin>)>> {
        let args = Args::parse();
        let plugins = DashMap::new();
        for plugin in list.into_iter() {
            let url = if plugin.is_relative_download_url() {
                plugin.build_url_with_console(&args.console)
            } else {
                plugin.url.clone()
            };

            let plugin_instance = match NetworkPlugin(url.clone()).async_try_into().await {
                Ok(instance) => instance,
                Err(e) => {
                    log::error!(
                        "plugin {} load failed: {}, download url: {}",
                        plugin.name,
                        e,
                        url
                    );
                    continue;
                }
            };
            plugins.insert(plugin.name.clone(), (plugin, plugin_instance));
        }

        Ok(plugins)
    }

    async fn fetch_plugins() -> anyhow::Result<Vec<PluginConfig>> {
        INNER_HTTP_CLIENT.fetch_plugins().await
    }

    const INTERVAL: Duration = Duration::from_secs(5);

    fn watch() {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Self::INTERVAL);
            loop {
                interval.tick().await;
                let list = Self::fetch_plugins().await;

                let list = match list {
                    Ok(list) => list,
                    Err(e) => {
                        log::error!("fetch plugins error: {}", e);
                        continue;
                    }
                };

                let hash = md5::compute(serde_json::to_string(&list).unwrap());
                let hash = format!("{:x}", hash);

                let old_plugins = PLUGINS.get().unwrap();

                if hash == *old_plugins.hash.read().await {
                    log::debug!("plugins not changed, wait next interval");
                    continue;
                }

                log::info!("loaded {} plugins", list.len());

                let new_plugins = Self::process_plugins(list).await.unwrap();
                {
                    old_plugins
                        .plugins
                        .retain(|name, _| new_plugins.contains_key(name));
                    new_plugins.into_iter().for_each(|(name, plugin)| {
                        old_plugins.plugins.insert(name, plugin);
                    });

                    *old_plugins.hash.write().await = hash;
                }
            }
        });
    }

    /// 调用插件
    pub async fn execute(
        &self,
        configured_plugin: &ConfiguredPlugin,
        context: &HttpContext,
    ) -> anyhow::Result<()> {
        match self.plugins.get(&configured_plugin.name) {
            Some(plugin) => plugin
                .1
                .execute(context, &configured_plugin.config)
                .await
                .map_err(|e| anyhow::anyhow!(e)),
            None => bail!(
                "plugin {} not found in plugin factory",
                &configured_plugin.name
            ),
        }
    }
}
