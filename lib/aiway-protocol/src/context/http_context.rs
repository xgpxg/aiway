use crate::context::Route;
use bytes::Bytes;
use dashmap::DashMap;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::sync::Arc;

/// 网关HTTP上下文
#[derive(Debug, Default)]
pub struct HttpContext {
    /// 匹配到路由配置信息
    route: Option<Arc<Route>>,
    /// 路由目标地址，可以是域名或IP(包含协议头)，由负载均衡器设置
    routing_url: Option<String>,
    /// 响应状态码
    pub response_status: Option<u16>,
    /// 响应时间戳
    pub response_ts: Option<i64>,
    /// 内部扩展数据
    pub inner_state: InnerState,
    /// 自定义的扩展数据
    pub state: DashMap<String, Value>,
}

impl HttpContext {
    pub fn set_route(&mut self, route: Arc<Route>) {
        self.route = route.into();
    }

    pub fn get_route(&self) -> Option<Arc<Route>> {
        self.route.clone()
    }

    pub fn set_routing_url(&mut self, url: String) {
        self.routing_url = url.into();
    }

    pub fn get_routing_url(&self) -> Option<&String> {
        self.routing_url.as_ref()
    }

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

#[derive(Debug, Default)]
pub struct InnerState(pub DashMap<String, Value>);

impl InnerState {
    #[cfg(feature = "model")]
    const MODEL_PROVIDER: &'static str = "model_proxy:provider";
    #[cfg(feature = "model")]
    pub fn get_model_provider(&self) -> Option<crate::model::Provider> {
        self.0.get(Self::MODEL_PROVIDER).and_then(|v| {
            serde_json::from_value(v.value().clone())
                .expect("Failed to deserialize model provider value")
        })
    }
    #[cfg(feature = "model")]
    pub fn set_model_provider(&self, provider: crate::model::Provider) {
        self.0.insert(
            Self::MODEL_PROVIDER.to_string(),
            serde_json::to_value(provider).expect("Failed to serialize model provider value"),
        );
    }

    pub fn set_temp_body(&self, body: Bytes) {
        self.0.insert(
            "temp_body".to_string(),
            serde_json::from_slice(body.as_ref()).unwrap(),
        );
    }

    pub fn get_temp_body(&self) -> Option<Bytes> {
        self.0.get("temp_body").map(|v| {
            Bytes::copy_from_slice(serde_json::to_vec(&v.value().clone()).unwrap().as_slice())
        })
    }
}
