//! # 模型代理
//! 需要尽量兼容OpenAI格式，部分场景可适当扩展
//!
//! 整体执行流程：
//! 网关 → model-proxy → 获取提供商 → 模型名称映射 → 请求参数转换 → 调用提供商 → 响应参数转换 → 返回结果
//!
use crate::model_proxy::proxy::ModelError;
use crate::model_proxy::proxy::client::Client;
use aiway_protocol::context::HttpContext;
use bytes::Bytes;
use dashmap::DashMap;
use logging::log;
use reqwest::Response;
use std::sync::LazyLock;

pub struct Proxy {
    /// (模型名称, 提供商名称) -> Client实例
    clients: DashMap<(String, String), Client>,
}

static PROXY: LazyLock<Proxy> = LazyLock::new(|| Proxy {
    clients: DashMap::new(),
});

macro_rules! get_or_create_client {
    ($model:expr, $provider:expr) => {{
        PROXY
            .clients
            .entry(($model.clone(), $provider.name.clone()))
            .or_insert_with(|| {
                log::info!("creating client for provider: {}", $provider.name);
                let client = Client::new($provider.api_key.clone());
                client
            })
    }};
}

impl Proxy {
    /// 移除某个模型下的所有Client实例
    ///
    /// 仅当模型配置发生变更（新增、修改、删除、提供商变更）时才需要调用此方法
    pub fn remove_clients(model_name: &str) {
        let model_name = model_name.to_string();
        PROXY.clients.retain(|(model, _), _| *model != model_name);
    }

    pub async fn call(body: Option<Bytes>, ctx: &mut HttpContext) -> Result<Response, ModelError> {
        if body.is_none() {
            return Err(ModelError::Parse("body is empty".to_string()));
        }
        let model = ctx.get_proxy_model_name().unwrap();
        let provider = ctx.get_proxy_model_provider().unwrap();

        let client = get_or_create_client!(model, &provider);
        let response = client
            .post(&provider.api_url, body.unwrap_or_default(), None)
            .await?;
        Ok(response)
    }
}
