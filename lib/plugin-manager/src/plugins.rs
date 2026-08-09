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

use crate::CONSOLE;
use crate::client::INNER_HTTP_CLIENT;
use crate::wasm::{AsyncTryInto, NetworkPlugin, Outcome, Plugin, PluginError};
use aiway_protocol::context::HttpContext;
use aiway_protocol::gateway::Plugin as PluginConfig;
use aiway_protocol::gateway::plugin::ConfiguredPlugin;
use dashmap::DashMap;
use logging::log;
use serde_json::Value;
use std::collections::HashSet;
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
        let plugins = DashMap::new();
        for plugin in list.into_iter() {
            let url = if plugin.is_relative_download_url() {
                plugin.build_url_with_console(CONSOLE.get().unwrap())
            } else {
                plugin.url.clone()
            };

            let plugin_instance = match (NetworkPlugin {
                url: url.clone(),
                checksum: plugin.checksum.clone(),
            })
            .async_try_into()
            .await
            {
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

                // 控制台当前插件名集合，用于剔除已下线插件
                let names: HashSet<String> = list.iter().map(|p| p.name.clone()).collect();

                let new_plugins = Self::process_plugins(list).await.unwrap();
                {
                    // 只删除控制台已不存在的插件；下载失败的插件保留旧实例，避免误下线
                    old_plugins.plugins.retain(|name, _| names.contains(name));
                    new_plugins.into_iter().for_each(|(name, plugin)| {
                        old_plugins.plugins.insert(name, plugin);
                    });

                    *old_plugins.hash.write().await = hash;
                }
            }
        });
    }

    /// 将插件配置注入 HttpContext，供 WASM 侧 `host_config` 宿主函数读取
    fn inject_config(ctx: &HttpContext, config: &Value) {
        ctx.insert_any_state(
            HttpContext::PLUGIN_CONFIG,
            serde_json::to_string(config).unwrap_or_default(),
        );
    }

    pub async fn on_request(
        configured_plugin: &ConfiguredPlugin,
        ctx: &mut HttpContext,
    ) -> Result<Outcome, PluginError> {
        match PLUGINS.get().unwrap().plugins.get(&configured_plugin.name) {
            Some(plugin) => {
                Self::inject_config(ctx, &configured_plugin.config);
                plugin.1.on_request(ctx).await
            }
            None => Err(PluginError::NotFound(configured_plugin.name.clone())),
        }
    }

    pub async fn on_request_body(
        configured_plugin: &ConfiguredPlugin,
        ctx: &mut HttpContext,
    ) -> Result<Outcome, PluginError> {
        match PLUGINS.get().unwrap().plugins.get(&configured_plugin.name) {
            Some(plugin) => {
                Self::inject_config(ctx, &configured_plugin.config);
                plugin.1.on_request_body(ctx).await
            }
            None => Err(PluginError::NotFound(configured_plugin.name.clone())),
        }
    }

    pub async fn on_response(
        configured_plugin: &ConfiguredPlugin,
        ctx: &mut HttpContext,
    ) -> Result<Outcome, PluginError> {
        match PLUGINS.get().unwrap().plugins.get(&configured_plugin.name) {
            Some(plugin) => {
                Self::inject_config(ctx, &configured_plugin.config);
                plugin.1.on_response(ctx).await
            }
            None => Err(PluginError::NotFound(configured_plugin.name.clone())),
        }
    }

    pub async fn on_response_body(
        configured_plugin: &ConfiguredPlugin,
        ctx: &mut HttpContext,
    ) -> Result<Outcome, PluginError> {
        match PLUGINS.get().unwrap().plugins.get(&configured_plugin.name) {
            Some(plugin) => {
                Self::inject_config(ctx, &configured_plugin.config);
                plugin.1.on_response_body(ctx).await
            }
            None => Err(PluginError::NotFound(configured_plugin.name.clone())),
        }
    }

    pub async fn on_logging(configured_plugin: &ConfiguredPlugin, ctx: &mut HttpContext) {
        if let Some(plugin) = PLUGINS.get().unwrap().plugins.get(&configured_plugin.name) {
            Self::inject_config(ctx, &configured_plugin.config);
            let _ = plugin.1.on_logging(ctx).await;
        }
    }
}
