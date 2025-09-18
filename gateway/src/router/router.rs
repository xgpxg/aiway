//! # 路由
//! 负责从配置中心加载路由表并缓存。
//!
//! 实现流程：
//! - 初始化时，从配置中心获取配置key为`routes`的配置项
//! - 反序列化为[`Vec<Route>`]
//! - 缓存路由表
//! - 监听配置`routes.yaml`变更，重写获取路由表并缓存
//!
use crate::constants;
use conreg_client::AppConfig;
use protocol::gateway::Route;
use std::sync::{Arc, LazyLock, RwLock};

pub struct Router {
    /// 路由表
    routes: Arc<RwLock<matchit::Router<Arc<Route>>>>,
}

pub static ROUTER: LazyLock<Router> = LazyLock::new(Router::load);

impl Router {
    pub fn load() -> Self {
        let list = Self::fetch_routes();

        let mut routes = matchit::Router::new();

        for route in list.into_iter() {
            routes.insert(route.path.clone(), Arc::new(route)).unwrap();
        }

        Self::watch();

        Self {
            routes: Arc::new(RwLock::new(routes)),
        }
    }

    fn fetch_routes() -> Vec<Route> {
        // 从配置中心哪路由表
        let mut routes = AppConfig::get::<Vec<Route>>("routes");
        log::info!("路由表: {:?}", routes);

        routes.unwrap_or_default()
    }

    fn watch() {
        AppConfig::add_listener(constants::ROUTES_CONFIG_ID, |_| {
            let list = Self::fetch_routes();

            let mut routes = matchit::Router::new();

            for route in list.into_iter() {
                routes.insert(route.path.clone(), Arc::new(route)).unwrap();
            }
            *ROUTER.routes.write().unwrap() = routes;
        });
    }

    pub fn matches(&self, path: &str) -> Option<Arc<Route>> {
        if let Ok(routes) = self.routes.read() {
            if let Ok(result) = routes.at(path) {
                return Some(result.value.clone());
            }
        }
        None
    }
}
