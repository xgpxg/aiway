use crate::SV;
use crate::gateway::route::Route;
use bytes::Bytes;
use dashmap::DashMap;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::sync::Arc;

/// 请求上下文
///
/// - 贯穿整个请求流程，同一个请求中应该仅持有一份数据。
/// - 不应该被多次创建，仅在请求阶段构建一次。
/// - 不需要序列化和反序列化
///
#[derive(Debug, Default)]
pub struct RequestContext {
    /// 请求ID
    pub request_id: String,
    /// 收到请求的时间戳，毫秒
    pub request_ts: i64,
    /// 请求方法：`GET` | `POST` | `PUT` | `DELETE` | `PATCH` | `OPTIONS` | `HEAD`
    pub method: SV<String>,
    /// Host
    ///
    /// http1.1及以前版本，取Header里的Host，
    /// http2中，取:authority
    pub host: String,
    /// 请求路径。
    pub path: SV<String>,
    /// 请求头
    pub headers: DashMap<String, String>,
    /// 请求参数
    pub query: DashMap<String, String>,
    /// 请求体
    pub body: SV<Bytes>,
    /// 扩展数据
    pub state: DashMap<String, Value>,
    /// 路由配置信息
    ///
    /// 路由由网关根据当前请求的path匹配得到，通常情况下，路由不应该手动修改。
    /// 由于Route是网关级别的配置，对全局有效，所以使用Arc来保持共享状态。
    pub route: SV<Arc<Route>>,
    /// 路由目标地址，可以是域名或IP，由负载均衡Fairing设置
    pub routing_url: SV<String>,
    // /// 实际路由路径
    // ///
    // /// 默认为网关接收到的原始路径，可以通过插件改写。
    // pub routing_path: SV<String>,
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

    // pub fn set_routing_path(&self, path: String) {
    //     self.routing_path.set(path);
    // }
    // pub fn get_routing_path(&self) -> Option<&String> {
    //     self.routing_path.get()
    // }

    pub fn insert_state<T: Serialize>(&self, key: &str, value: T) {
        self.state.insert(
            key.to_string(),
            serde_json::to_value(value).expect("Failed to serialize state value"),
        );
    }
    pub fn get_state<T: DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<T>, serde_json::Error> {
        self.state
            .get(key)
            .map(|v| serde_json::from_value(v.clone()))
            .transpose()
    }
    pub fn remove_state(&self, key: &str) {
        self.state.remove(key);
    }
}
