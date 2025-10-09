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

use crate::router::client::INNER_HTTP_CLIENT;
use dashmap::DashMap;
use globset::{Glob, GlobSet, GlobSetBuilder};
use protocol::gateway::{HttpContext, Route};
use std::collections::BTreeMap;
use std::process::exit;
use std::sync::{Arc, OnceLock, RwLock};
use std::time::Duration;

pub struct Router {
    /// 路由表
    routes: Arc<RwLock<Vec<Arc<Route>>>>,
    /// 路由匹配器
    matcher: Arc<RwLock<GlobSet>>,
}

pub static ROUTER: OnceLock<Router> = OnceLock::new();

impl Router {
    pub async fn init() {
        if let Err(e) = Self::load().await {
            log::error!("{}", e);
            exit(1)
        }
    }
    async fn load() -> anyhow::Result<()> {
        let routes = Self::fetch_routes()
            .await?
            .into_iter()
            .map(Arc::new)
            .collect::<Vec<_>>();

        log::info!("loaded {} routes", routes.len());

        let matcher = Self::build_matcher(&routes);

        let router = Router {
            routes: Arc::new(RwLock::new(routes)),
            matcher: Arc::new(RwLock::new(matcher)),
        };

        ROUTER.get_or_init(|| router);

        Self::watch();

        Ok(())
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

    async fn fetch_routes() -> anyhow::Result<Vec<Route>> {
        INNER_HTTP_CLIENT.fetch_routes().await
    }

    const INTERVAL: Duration = Duration::from_secs(5);
    fn watch() {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Self::INTERVAL);

            loop {
                interval.tick().await;

                log::debug!("{}", "reloading routes from console");
                let routes = Self::fetch_routes().await;

                let routes = match routes {
                    Ok(routes) => routes,
                    Err(e) => {
                        log::error!("{}", e);
                        continue;
                    }
                };

                let old_routes: Vec<Route> = {
                    let guard = ROUTER.get().unwrap().routes.read().unwrap();
                    guard.iter().map(|r| (**r).clone()).collect()
                };

                let old = serde_json::to_string(&old_routes).unwrap();
                let new = serde_json::to_string(&routes).unwrap();

                if old == new {
                    log::debug!("routes not changed, wait next interval");
                    continue;
                }

                log::info!("loaded {} routes", routes.len());

                let routes = routes.into_iter().map(Arc::new).collect::<Vec<_>>();

                let matcher = Self::build_matcher(&routes);

                {
                    *ROUTER.get().unwrap().routes.write().unwrap() = routes;
                    *ROUTER.get().unwrap().matcher.write().unwrap() = matcher;
                }
            }
        });
    }

    pub fn matches(&self, context: Arc<HttpContext>) -> Option<Arc<Route>> {
        if let Ok(matcher) = self.matcher.read() {
            let indexes = matcher.matches(context.request.get_path());
            let host = &context.request.host;
            let query = &context.request.query;
            for index in indexes.iter() {
                if let Ok(routes) = self.routes.read() {
                    // 先按路径匹配，减小范围
                    if let Some(route) = routes.get(*index) {
                        // Host匹配
                        if !Self::matches_host(&route, host) {
                            continue;
                        }
                        // Header匹配
                        if !Self::matches_headers(route, &context) {
                            continue;
                        }
                        // Query匹配
                        if !Self::matches_query(route, query) {
                            continue;
                        }

                        log::debug!("matched route: {:?}", route);
                        return Some(route.clone());
                    }
                }
            }
        }
        None
    }
    fn matches_host(route: &Route, host: &String) -> bool {
        route.host.is_none() || route.host.as_ref() == Some(host)
    }

    fn matches_headers(route: &Route, context: &HttpContext) -> bool {
        route
            .header
            .iter()
            .all(|(key, value)| context.request.get_header(key).as_ref() == Some(value))
    }

    fn matches_query(route: &Route, query: &DashMap<String, String>) -> bool {
        route
            .query
            .iter()
            .all(|(key, value)| query.get(key).map(|v| v.value() == value).unwrap_or(false))
    }
}
