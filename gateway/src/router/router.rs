//! # 路由
//! 负责从配置中心加载路由表并缓存。
//!
//! 实现流程：
//! - 初始化时，从配置中心获取配置key为`routes`的配置项
//! - 反序列化为[`Vec<Route>`]
//! - 缓存路由表
//! - 监听配置`routes.yaml`变更，重写获取路由表并缓存
//!
//! TODO 以上内容需重写
//!
//! # 路由
//! 负责从控制台加载路由表并缓存。
//!
//! 实现流程：
//! - 初始化时，尝试从控制台的`GET /api/v1/gateway/routes`端点获取路由表。
//! - 如果控制台无法连接，则退出，禁止启动。
//! - 反序列化响应结果为[`Vec<Route>`]
//! - 缓存路由表到内存以及本地。
//! - 启动定时任务，每5秒从控制台拉取路由表，校验hash值，如果不一致则更新本地路由表。
//!
//! 路由定义：[`Route`]
//!
//!
//!

use crate::constants;
use conreg_client::AppConfig;
use globset::{Glob, GlobSet, GlobSetBuilder};
use protocol::gateway::Route;
use std::sync::{Arc, LazyLock, RwLock};

pub struct Router {
    /// 路由表
    routes: Arc<RwLock<Vec<Arc<Route>>>>,
    /// 路由匹配器
    matcher: Arc<RwLock<GlobSet>>,
}

pub static ROUTER: LazyLock<Router> = LazyLock::new(Router::load);

impl Router {
    pub fn load() -> Self {
        let routes = Self::fetch_routes()
            .into_iter()
            .map(Arc::new)
            .collect::<Vec<_>>();
        let matcher = Self::build_matcher(&routes);

        Self::watch();

        Self {
            routes: Arc::new(RwLock::new(routes)),
            matcher: Arc::new(RwLock::new(matcher)),
        }
    }

    fn build_matcher(routes: &[Arc<Route>]) -> GlobSet {
        let mut builder = GlobSetBuilder::new();
        for route in routes {
            // 匹配规则为：前缀+路径
            let pattern = format!(
                "{}{}",
                route.prefix.as_deref().unwrap_or_default(),
                route.path
            );
            builder.add(Glob::new(&pattern).unwrap());
        }
        builder.build().unwrap()
    }

    fn fetch_routes() -> Vec<Route> {
        // 从配置中心哪路由表
        let mut routes = AppConfig::get::<Vec<Route>>("routes").unwrap_or_default();
        log::info!("fetched {} routes", routes.len());
        log::debug!("routes: {:?}", routes);

        routes
    }

    fn watch() {
        AppConfig::add_listener(constants::ROUTES_CONFIG_ID, |_| {
            let routes = Self::fetch_routes()
                .into_iter()
                .map(Arc::new)
                .collect::<Vec<_>>();

            let matcher = Self::build_matcher(&routes);

            {
                *ROUTER.routes.write().unwrap() = routes;
            }
            {
                *ROUTER.matcher.write().unwrap() = matcher;
            }
        });
    }

    pub fn matches(&self, path: &str) -> Option<Arc<Route>> {
        if let Ok(matcher) = self.matcher.read() {
            let indexes = matcher.matches(path);
            // 多个匹配，优先选择第一个
            // TODO header、query匹配
            if let Some(&index) = indexes.first() {
                if let Ok(routes) = self.routes.read() {
                    return routes.get(index).cloned();
                }
            }
        }
        None
    }
}
