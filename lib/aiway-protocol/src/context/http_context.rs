use crate::context::Route;
use crate::context::parts::SerdeParts;
#[cfg(feature = "model")]
use crate::model::Provider;
use bincode;
use dashmap::DashMap;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// 网关HTTP上下文
#[derive(Debug)]
pub struct HttpContext {
    /// 路由信息
    routing: Routing,
    /// 运行时数据，必须是可序列化的。
    state: State,
    /// 运行时数据，可以是任何数据，在请求生命周期内有效。
    any_state: AnyState,
}

#[derive(Debug, Default)]
pub struct Routing {
    /// 匹配到的路由配置信息
    route: Option<Arc<Route>>,
    /// 路由目标地址，可以是域名或IP(包含协议头)，由负载均衡器设置。
    target: Option<String>,
}

#[derive(Debug)]
pub struct State(DashMap<String, Vec<u8>>);

impl Default for State {
    fn default() -> Self {
        let data = DashMap::new();

        // 插入请求 ID
        let request_id = uuid::Uuid::new_v4().to_string();
        let encoded_id = bincode::serialize(&request_id).unwrap();
        data.insert(HttpContext::REQUEST_ID.to_string(), encoded_id);

        // 插入请求时间戳
        let request_ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        let encoded_ts = bincode::serialize(&request_ts).unwrap();
        data.insert(HttpContext::REQUEST_TS.to_string(), encoded_ts);

        State(data)
    }
}

impl State {
    pub fn insert<T: Serialize>(&self, key: &str, value: T) -> Option<Vec<u8>> {
        let encoded = bincode::serialize(&value).unwrap();
        self.0.insert(key.to_string(), encoded)
    }

    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.0
            .get(key)
            .and_then(|v| bincode::deserialize::<T>(v.value()).ok())
    }

    pub fn remove(&self, key: &str) -> Option<Vec<u8>> {
        self.0.remove(key).map(|(_, v)| v)
    }

    pub fn exists(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }
}

#[derive(Debug, Default)]
pub struct AnyState {
    data: DashMap<String, Arc<dyn Any + Send + Sync>>,
}

impl AnyState {
    pub fn insert<T: Any + Send + Sync>(
        &self,
        key: &str,
        value: T,
    ) -> Option<Arc<dyn Any + Send + Sync>> {
        self.data.insert(key.to_string(), Arc::new(value))
    }

    pub fn get<T: Any + Send + Sync>(&self, key: &str) -> Option<Arc<T>> {
        self.data
            .get(key)
            .and_then(|v| v.clone().downcast::<T>().ok())
    }

    pub fn remove<T: Any + Send + Sync>(&self, key: &str) -> Option<Arc<T>> {
        self.data
            .remove(key)
            .and_then(|(_, v)| v.downcast::<T>().ok())
    }

    pub fn exists(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }
}

impl Default for HttpContext {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpContext {
    /// 请求ID
    pub const REQUEST_ID: &'static str = ":req:id";
    /// 请求时间戳，毫秒
    pub const REQUEST_TS: &'static str = ":req:ts";
    /// 请求的Parts（原始值，未经过网关处理），格式为[http::request::Parts]。
    /// 在请求生命周期内始终有效，不可变。
    pub const REQUEST_RAW_PARTS: &'static str = ":req:raw:parts";
    /// 最终响应给客户端的Parts，格式为[http::response::Parts]
    pub const RESPONSE_SERDE_PARTS: &'static str = ":resp:parts";
    /// 响应的Body大小
    pub const RESPONSE_BODY_SIZE: &'static str = ":resp:parts:body_size";
    /// 是否sse
    pub const IS_SSE: &'static str = ":resp:is_sse";
    /// 是否websocket
    pub const IS_WEBSOCKET: &'static str = ":resp:is_ws";
    /// 模型名称，仅适用于模型插件
    pub const MODEL_PROXY_MODEL: &'static str = ":model_proxy:model";
    /// 模型提供商，仅适用于模型插件
    pub const MODEL_PROXY_PROVIDER: &'static str = ":model_proxy:provider";
    /// 模型调用 prompt token 数量
    pub const MODEL_USAGE_PROMPT_TOKENS: &'static str = ":model:usage:prompt_tokens";
    /// 模型调用 completion token 数量
    pub const MODEL_USAGE_COMPLETION_TOKENS: &'static str = ":model:usage:completion_tokens";
    /// 模型调用 total token 数量
    pub const MODEL_USAGE_TOTAL_TOKENS: &'static str = ":model:usage:total_tokens";
    /// 模型调用首 Token 响应时间（毫秒）
    pub const MODEL_TTFT_MS: &'static str = ":model:ttft_ms";
    /// 响应状态码
    pub const RESPONSE_STATUS_CODE: &'static str = ":resp:status_code";
    pub fn new() -> Self {
        Self {
            routing: Default::default(),
            state: Default::default(),
            any_state: Default::default(),
        }
    }

    #[inline]
    pub fn set_route(&mut self, route: Arc<Route>) {
        self.routing.route = Some(route);
    }

    #[inline]
    pub fn get_route(&self) -> Option<Arc<Route>> {
        self.routing.route.clone()
    }

    #[inline]
    pub fn set_routing_url(&mut self, url: String) {
        self.routing.target = Some(url);
    }

    #[inline]
    pub fn get_routing_url(&self) -> Option<&String> {
        self.routing.target.as_ref()
    }

    #[inline]
    pub fn insert_state<T: Serialize>(&self, key: &str, value: T) {
        self.state.insert(key, value);
    }

    #[inline]
    pub fn get_state<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.state.get(key)
    }

    #[inline]
    pub fn remove_state(&self, key: &str) {
        self.state.remove(key);
    }

    #[inline]
    pub fn insert_any_state<T: Any + Send + Sync>(&self, key: &str, value: T) {
        self.any_state.insert(key, value);
    }

    #[inline]
    pub fn get_any_state<T: Any + Send + Sync>(&self, key: &str) -> Option<Arc<T>> {
        self.any_state.get(key)
    }

    #[inline]
    pub fn remove_any_state<T: Any + Send + Sync>(&self, key: &str) {
        self.any_state.remove::<T>(key);
    }

    #[inline]
    pub fn exists_any_state(&self, key: &str) -> bool {
        self.any_state.exists(key)
    }

    /// 获取请求ID
    pub fn request_id(&self) -> String {
        //SAFE
        self.state.get(Self::REQUEST_ID).unwrap()
    }

    /// 获取请求时间戳（网关收到请求的时间）
    pub fn request_ts(&self) -> i64 {
        //SAFE
        self.state.get(Self::REQUEST_TS).unwrap()
    }

    pub fn request_raw_parts(&self) -> Option<SerdeParts> {
        self.state.get(Self::REQUEST_RAW_PARTS)
    }

    /// 获取请求的模型名称
    ///
    /// 使用限制：仅在模型插件时可用
    #[cfg(feature = "model")]
    #[inline]
    pub fn get_proxy_model_name(&self) -> Option<String> {
        self.state.get(Self::MODEL_PROXY_MODEL)
    }

    /// 获取命中的模型代理商
    ///
    /// 使用限制：仅在模型插件时可用
    #[cfg(feature = "model")]
    #[inline]
    pub fn get_proxy_model_provider(&self) -> Option<Provider> {
        self.state.get(Self::MODEL_PROXY_PROVIDER)
    }

    #[inline]
    pub fn is_sse(&self) -> bool {
        self.state.exists(Self::IS_SSE)
    }

    #[inline]
    pub fn is_websocket(&self) -> bool {
        self.state.exists(Self::IS_WEBSOCKET)
    }
}
