use crate::context::request_context::RequestContext;
use crate::context::response_context::ResponseContext;
use dashmap::DashMap;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

/// HTTP上下文
#[derive(Debug, Default)]
pub struct HttpContext {
    /// 请求上下文，在请求阶段构建
    pub request: RequestContext,
    /// 响应上下文，在构建请求上下文时同步构建，在响应阶段更新
    pub response: ResponseContext,
    /// 内部扩展数据
    pub inner_state: InnerState,
    /// 自定义的扩展数据
    pub state: DashMap<String, Value>,
}

impl HttpContext {
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
}
