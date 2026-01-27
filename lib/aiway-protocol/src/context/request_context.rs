use crate::SV;
use crate::context::route::Route;
use bytes::Bytes;
use dashmap::DashMap;
use std::sync::Arc;

/// 请求上下文
#[derive(Debug, Default)]
pub struct RequestContext {
    /// 请求ID
    pub request_id: String,
    /// 收到请求的时间戳，毫秒
    pub request_ts: i64,
    /// 请求方法：`GET` | `POST` | `PUT` | `DELETE` | `PATCH` | `OPTIONS` | `HEAD`
    pub method: SV<String>,
    /// Host
    pub host: String,
    /// 请求路径，不含参数。
    pub path: SV<String>,
    /// 请求头
    pub headers: DashMap<String, String>,
    /// 请求的Query参数
    pub query: DashMap<String, String>,
    /// 请求体
    pub body: SV<Bytes>,
    /// 路由配置信息
    pub route: SV<Arc<Route>>,
    /// 路由目标地址，可以是域名或IP(包含协议头)，由负载均衡器设置
    pub routing_url: SV<String>,
}

impl RequestContext {
    pub fn get_request_ts(&self) -> i64 {
        self.request_ts
    }

    pub fn get_host(&self) -> &str {
        &self.host
    }
    pub fn get_method(&self) -> Option<&str> {
        self.method.get().map(|s| s.as_str())
    }
    pub fn set_path(&self, path: &str) {
        self.path.set(path.to_string());
    }

    pub fn get_path(&self) -> String {
        self.path.get().cloned().unwrap_or_default()
    }

    pub fn insert_header(&self, key: &str, value: &str) {
        self.headers.insert(key.to_string(), value.to_string());
    }

    pub fn get_header(&self, key: &str) -> Option<String> {
        self.headers.get(key).map(|v| v.value().clone())
    }

    pub fn remove_header(&self, key: &str) {
        self.headers.remove(key);
    }

    pub fn insert_query(&self, name: &str, value: &str) {
        self.query.insert(name.to_string(), value.to_string());
    }

    pub fn get_query(&self, name: &str) -> Option<String> {
        self.query.get(name).map(|v| v.value().clone())
    }

    pub fn set_body(&self, body: Bytes) {
        self.body.set(body)
    }

    pub fn get_body(&self) -> Option<&Bytes> {
        self.body.get()
    }

    pub fn set_route(&self, route: Arc<Route>) {
        self.route.set(route);
    }

    pub fn get_route(&self) -> Option<&Arc<Route>> {
        self.route.get()
    }

    pub fn set_routing_url(&self, url: String) {
        self.routing_url.set(url);
    }

    pub fn get_routing_url(&self) -> Option<&String> {
        self.routing_url.get()
    }

}
