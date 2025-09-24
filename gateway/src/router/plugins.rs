//! # 插件
//! 负责从控制台加载所有已启用的插件并缓存。
//!
//! 实现流程：
//! - 初始化时，尝试从控制台的`GET /api/v1/gateway/plugins`端点获取插件列表。
//! - 如果控制台无法连接，则退出，禁止启动。
//! - 缓存插件列表到内存以及本地。
//! - 启动定时任务，每5秒从控制台拉取插件列表，校验hash值，如果不一致则更新本地插件列表。
//!
use crate::constants;
use conreg_client::AppConfig;
use dashmap::{DashMap, DashSet};
use loadbalance::LoadBalance;
use plugin::{AsyncTryInto, NetworkPlugin, Plugin, PluginManager};
use protocol::gateway::plugin::PluginPhase;
use protocol::gateway::service::LbStrategy;
use protocol::gateway::{Plugin as PluginConfig, Route};
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;

pub struct Plugins {
    /// 全局过滤器插件（请求阶段）
    pub global_pre_filter_plugins: Arc<RwLock<Vec<(PluginConfig, Box<dyn Plugin>)>>>,
    /// 全局过滤器插件（响应阶段）
    pub global_post_filter_plugins: Arc<RwLock<Vec<(PluginConfig, Box<dyn Plugin>)>>>,
    /// 路由过滤器插件（请求阶段）
    pub pre_filter_plugins: Arc<RwLock<Vec<(PluginConfig, Box<dyn Plugin>)>>>,
    /// 路由过滤器插件（响应阶段）
    pub post_filter_plugins: Arc<RwLock<Vec<(PluginConfig, Box<dyn Plugin>)>>>,
}

pub static PLUGINS: OnceLock<Plugins> = OnceLock::new();

macro_rules! distribute_plugin {
    ($phase:expr, $plugin:expr, $instance:expr, { $($variant:ident => $vec:expr),* $(,)? }) => {
        match $phase {
            $(
                PluginPhase::$variant => {
                    $vec.push(($plugin, $instance));
                }
            )*
        }
    };
}
impl Plugins {
    /// 初始化插件
    ///
    /// 该方法为异步的，是因为插件需要从远程加载，需要异步的，不然在插件同步时会阻塞线程。
    pub async fn init() {
        let plugins = Plugins::load().await;
        PLUGINS.get_or_init(|| plugins);
    }

    pub async fn load() -> Self {
        let list = Self::fetch_plugins();

        let mut global_pre_filter_plugins = Vec::new();
        let mut global_post_filter_plugins = Vec::new();
        let mut pre_filter_plugins = Vec::new();
        let mut post_filter_plugins = Vec::new();

        for plugin in list.into_iter() {
            let plugin_instance = match NetworkPlugin(plugin.url.clone()).async_try_into().await {
                Ok(instance) => instance,
                Err(e) => {
                    log::error!("plugin {} load failed: ", e);
                    continue;
                }
            };
            distribute_plugin!(
                plugin.phase,
                plugin,
                plugin_instance,
                {
                    GlobalPre => global_pre_filter_plugins,
                    GlobalPost => global_post_filter_plugins,
                    Pre => pre_filter_plugins,
                    Post => post_filter_plugins
                }
            );
        }

        Self::watch();

        Self {
            global_pre_filter_plugins: Arc::new(RwLock::new(global_pre_filter_plugins)),
            global_post_filter_plugins: Arc::new(RwLock::new(global_post_filter_plugins)),
            pre_filter_plugins: Arc::new(RwLock::new(pre_filter_plugins)),
            post_filter_plugins: Arc::new(RwLock::new(post_filter_plugins)),
        }
    }

    fn fetch_plugins() -> Vec<PluginConfig> {
        // 从配置中心拿插件
        let plugins = AppConfig::get::<Vec<PluginConfig>>("plugins").unwrap_or_default();
        log::info!("fetched {} plugins", plugins.len());
        log::debug!("plugins: {:?}", plugins);

        plugins
    }

    fn watch() {
        AppConfig::add_listener(constants::PLUGINS_CONFIG_ID, |_| {
            tokio::spawn(async move {
                let list = Self::fetch_plugins();

                // 全局实例
                let plugins = PLUGINS.get().unwrap();

                let mut global_pre_filter_plugins = Vec::new();
                let mut global_post_filter_plugins = Vec::new();
                let mut pre_filter_plugins = Vec::new();
                let mut post_filter_plugins = Vec::new();

                for plugin in list.into_iter() {
                    let plugin_instance =
                        match NetworkPlugin(plugin.url.clone()).async_try_into().await {
                            Ok(instance) => instance,
                            Err(e) => {
                                log::error!("plugin {} load failed: {}", plugin.name, e);
                                continue;
                            }
                        };
                    distribute_plugin!(
                        plugin.phase,
                        plugin,
                        plugin_instance,
                        {
                            GlobalPre => global_pre_filter_plugins,
                            GlobalPost => global_post_filter_plugins,
                            Pre => pre_filter_plugins,
                            Post => post_filter_plugins
                        }
                    );
                }

                {
                    let mut global_pre = plugins.global_pre_filter_plugins.write().await;
                    *global_pre = global_pre_filter_plugins;
                }
                {
                    let mut global_post = plugins.global_post_filter_plugins.write().await;
                    *global_post = global_post_filter_plugins;
                }
                {
                    let mut pre = plugins.pre_filter_plugins.write().await;
                    *pre = pre_filter_plugins;
                }
                {
                    let mut post = plugins.post_filter_plugins.write().await;
                    *post = post_filter_plugins;
                }
            });
        });
    }
}
