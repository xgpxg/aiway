//! # 路由
//! 负责从console加载路由表并缓存。
//!
//! 实现流程：
//! - 初始化时，调用控制台服务，从指定端点加载路由表
//! - 网关缓存路由表
//! - 引入配置中心客户端，监听指定配置`route-change-event.yml`
//! - console在修改路由配置，并持久化后，调用配置中心接口，修改`route-change-event.yml`配置，重新加载路由表
//! - 配置`route-change-event.yml`大概内容如下：
//! ```yaml
//! last-change: 毫秒时间戳
//! ```

use protocol::gateway::Route;
use std::sync::{Arc, LazyLock};

pub struct Router {
    /// 路由表
    routes: matchit::Router<Arc<Route>>,
}

pub static ROUTER: LazyLock<Router> = LazyLock::new(Router::load);

impl Router {
    pub fn load() -> Self {
        // 1. 通过负载均衡组件调用console
        // 2. 获取并缓存路由表
        // 3. 长轮询监听指定配置，实时更新

        // 测试数据
        let mut routes = matchit::Router::new();
        let mut route = Route::default();
        route.path = "/hello".into();
        route.service_id = "test-server".into();
        routes.insert(route.path.clone(), Arc::new(route)).unwrap();

        Self { routes }
    }

    pub fn add(&mut self, route: Route) {
        self.routes
            .insert(route.path.clone(), Arc::new(route))
            .unwrap();
    }

    pub fn matches(&self, path: &str) -> Option<Arc<Route>> {
        self.routes.at(path).ok().map(|res| res.value.clone())
    }
}
