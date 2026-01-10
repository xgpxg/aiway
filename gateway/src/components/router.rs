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

use crate::components::client::INNER_HTTP_CLIENT;
use dashmap::DashMap;
use aiway_protocol::gateway::{HttpContext, Route};
use std::process::exit;
use std::sync::{Arc, OnceLock, RwLock};
use std::time::Duration;

pub struct Router {
    /// 路由表
    routes: Arc<RwLock<Vec<Arc<Route>>>>,
    /// 路由匹配器
    matcher: Arc<RwLock<matchit::Router<Arc<Route>>>>,
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

    fn build_matcher(routes: &[Arc<Route>]) -> matchit::Router<Arc<Route>> {
        // 对routes排序，按照host长度降序、path长度降序、header个数降序、query个数降序的顺序排序
        // 这样保证更具体的优先匹配
        let mut routes = routes.iter().collect::<Vec<_>>();
        routes.sort_unstable_by(|a, b| {
            b.host
                .len()
                .cmp(&a.host.len())
                .then_with(|| b.path.len().cmp(&a.path.len()))
                .then_with(|| b.header.len().cmp(&a.header.len()))
                .then_with(|| b.query.len().cmp(&a.query.len()))
        });
        // 匹配器，key为路径
        let mut matcher = matchit::Router::new();

        for route in routes {
            // 这里理论上不会发生路径冲突，因为在控制台保存的时候已经验证了
            // 但为了避免错误，这里这里还是输出一下日志
            // 如果这里输出错误日志了，应该检查控制台的验证逻辑是否正确
            if let Err(e) = matcher.insert(route.match_path.clone(), route.clone()) {
                log::error!("build route matcher error: {}", e);
            }
        }
        matcher
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
                log::debug!("old routes: {}", old);
                log::debug!("new routes: {}", new);

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
        if let Ok(router) = self.matcher.read()
            && let Ok(result) = router.at(&context.request.get_path())
        {
            let route = result.value;
            // 再依次匹配 Host/Method/Header/Query
            if Self::match_host(route, context.request.get_host())
                && Self::match_method(route, context.request.get_method())
                && Self::match_host(route, &context.request.host)
                && Self::match_header(route, &context)
                && Self::match_query(route, &context.request.query)
            {
                return Some(route.clone());
            }
        }

        None
    }

    fn match_host(route: &Route, host: &str) -> bool {
        let route_host = &route.host;

        // 精确匹配
        // 因为大部分情况下Host配置的都是具体的，所以优先完全匹配，避免没必要的检查
        if route_host == host {
            return true;
        }

        // 匹配所有的
        if route_host == "*" {
            return true;
        }

        // 通配符匹配
        // 目前仅支持单个通配符，如 *.example.com
        if let Some(suffix) = route_host.strip_prefix('*') {
            // 避免匹配 *.example.com 匹配 *.example.com
            // 通配符应该只匹配子域名
            if host == &suffix[1..] {
                return false;
            }

            // 子域名匹配 (sub.example.com 匹配 *.example.com)
            if host.ends_with(suffix) {
                let prefix_len = host.len() - suffix.len();
                // 确保是完整的子域名
                if prefix_len > 0 {
                    return true;
                }
            }
        }

        false
    }

    fn match_method(route: &Route, method: Option<&str>) -> bool {
        route.methods.is_empty()
            || route
                .methods
                .iter()
                .any(|route_method| Some(route_method.as_str()) == method)
    }

    fn match_header(route: &Route, context: &HttpContext) -> bool {
        route
            .header
            .iter()
            .all(|(key, value)| context.request.get_header(key).as_ref() == Some(value))
    }

    fn match_query(route: &Route, query: &DashMap<String, String>) -> bool {
        route
            .query
            .iter()
            .all(|(key, value)| query.get(key).map(|v| v.value() == value).unwrap_or(false))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_matches() {
        let mut router = matchit::Router::new();
        router.insert(convert("/api/b/"), 1).unwrap();
        router.insert(convert("/api/**"), 2).unwrap();

        let matched = router.at("/api/b/").unwrap();
        assert_eq!(matched.value, &1);
    }

    fn convert(pattern: &str) -> String {
        let mut result = String::new();
        let mut param_count = 1;
        let mut chars = pattern.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '*' {
                // Check if it's a tailing "**" capturing all remaining path
                if chars.peek() == Some(&'*') {
                    chars.next(); // consume the second '*'
                    result.push_str("{*p}");
                } else {
                    // Single '*' - named parameter
                    result.push_str(&format!("{{p{}}}", param_count));
                    param_count += 1;
                }
            } else {
                result.push(ch);
            }
        }

        println!("{}", result);
        result
    }
}
