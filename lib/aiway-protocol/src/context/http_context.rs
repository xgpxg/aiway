use crate::context::Route;
use dashmap::DashMap;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// 网关HTTP上下文
#[derive(Debug)]
pub struct HttpContext {
    /// 请求ID
    request_id: String,
    /// 请求时间戳, 毫秒
    request_ts: i64,
    /// 匹配到路由配置信息
    route: Option<Arc<Route>>,
    /// 路由目标地址，可以是域名或IP(包含协议头)，由负载均衡器设置
    routing: Option<String>,
    /// 响应状态码
    pub response_status: Option<u16>,
    /// 响应时间戳
    pub response_ts: Option<i64>,
    /// 扩展数据
    pub state: State,
}

#[derive(Debug, Default)]
pub struct State {
    inner: DashMap<String, Value>,
}

impl State {
    pub fn insert<T: Serialize>(&self, key: &str, value: T) -> Option<Value> {
        self.inner
            .insert(key.to_string(), serde_json::to_value(value).unwrap())
    }

    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.inner
            .get(key)
            .and_then(|v| serde_json::from_value::<T>(v.clone()).ok())
    }

    pub fn remove(&self, key: &str) -> Option<Value> {
        self.inner.remove(key).map(|(_, v)| v)
    }
}

impl Default for HttpContext {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpContext {
    pub fn new() -> Self {
        Self {
            request_id: uuid::Uuid::new_v4().to_string(),
            request_ts: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64,
            route: None,
            routing: None,
            response_status: None,
            response_ts: None,
            state: Default::default(),
        }
    }
    #[inline]
    pub fn set_route(&mut self, route: Arc<Route>) {
        self.route = route.into();
    }
    #[inline]
    pub fn get_route(&self) -> Option<Arc<Route>> {
        self.route.clone()
    }
    #[inline]
    pub fn set_routing_url(&mut self, url: String) {
        self.routing = url.into();
    }
    #[inline]
    pub fn get_routing_url(&self) -> Option<&String> {
        self.routing.as_ref()
    }
}
