//! # 模型代理
//! 需要尽量兼容OpenAI格式，部分场景可适当扩展
//!
//! 整体执行流程：
//! 网关 → model-proxy → 获取提供商 → 模型名称映射 → 请求参数转换 → 调用提供商 → 响应参数转换 → 返回结果
//!
use crate::model_proxy::proxy::client::Client;
use crate::model_proxy::proxy::request::ModifyModelName;
use crate::model_proxy::proxy::response::{ModelError, ModelResponse};
use aiway_model_protocol::audio::{AudioSpeechParameters, AudioSpeechResponse};
use aiway_model_protocol::chat::{ChatCompletionChunkResponse, ChatCompletionParameters};
use aiway_model_protocol::image::{CreateImageParameters, EditImageParameters};
use aiway_protocol::common::constants::BAN_HEADERS;
use aiway_protocol::context::HttpContext;
use aiway_protocol::model::Provider;
use bytes::Bytes;
use dashmap::DashMap;
use logging::log;
use plugin_manager::PluginFactory;
use reqwest::Response;
use serde::Serialize;
use serde_json::Value;
use std::sync::LazyLock;
use tokio_stream::StreamExt;

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

    fn modify_model_name<R: ModifyModelName>(req: R, provider: &Provider) -> R {
        if let Some(target_model_name) = &provider.target_model_name
            && !target_model_name.is_empty()
        {
            log::debug!(
                "model name convert: {} -> {} ({})",
                req.get_source_model_name(),
                target_model_name,
                provider.name
            );
            req.modify_model_name(target_model_name)
        } else {
            req
        }
    }

    /// 对话补全
    pub async fn chat_completions(
        req: ChatCompletionParameters,
        provider: &Provider,
    ) -> Result<Response, ModelError> {
        let client = get_or_create_client!(req.model, provider);
        let req = Self::modify_model_name(req, provider);
        let response = client
            .post(&provider.api_url, serde_json::to_vec(&req).unwrap(), None)
            .await?;
        Ok(response)
    }

    /*    /// 文本转语音
    pub async fn audio_speech(
        req: AudioSpeechParameters,
        provider: &Provider,
        context: &HttpContext,
    ) -> Result<ModelResponse, ModelError> {
        let client = get_or_create_client!(req.model, provider);
        let req = Self::modify_model_name(req, provider);
        Self::convert_request(&req, provider, context).await?;

        let request_body = context.request.get_body().cloned().unwrap_or_default();
        let response = client.post(&provider.api_url, request_body, None).await?;

        Self::convert_response(response, provider, context).await?;

        Ok(ModelResponse::AudioSpeechResponse(
            context.response.get_status().unwrap_or_default(),
            context.response.headers.clone(),
            AudioSpeechResponse {
                bytes: context.response.body.take().unwrap(),
            },
        ))
    }

    /// 创建图像(文生图)
    pub async fn create_image(
        req: CreateImageParameters,
        provider: &Provider,
        context: &HttpContext,
    ) -> Result<ModelResponse, ModelError> {
        let client = get_or_create_client!(req.model.clone().unwrap_or_default(), provider);
        let req = Self::modify_model_name(req, provider);
        Self::convert_request(&req, provider, context).await?;

        let request_body = context.request.get_body().cloned().unwrap_or_default();

        let response = client.post(&provider.api_url, request_body, None).await?;

        Self::convert_response(response, provider, context).await?;

        let body = context.response.body.take().unwrap_or_default();
        let body = serde_json::from_slice(&body).map_err(|e| ModelError::Parse(e.to_string()))?;
        Ok(ModelResponse::CreateImageResponse(
            context.response.get_status().unwrap_or_default(),
            context.response.headers.clone(),
            body,
        ))
    }

    pub(crate) async fn edit_image(
        req: EditImageParameters,
        provider: &Provider,
        context: &HttpContext,
    ) -> Result<ModelResponse, ModelError> {
        let client = get_or_create_client!(req.model.clone().unwrap_or_default(), provider);
        let req = Self::modify_model_name(req, provider);
        Self::convert_request(&req, provider, context).await?;

        let request_body = context.request.get_body().cloned().unwrap_or_default();

        let response = client.post(&provider.api_url, request_body, None).await?;

        Self::convert_response(response, provider, context).await?;

        let body = context.response.body.take().unwrap_or_default();
        let body = serde_json::from_slice(&body).map_err(|e| ModelError::Parse(e.to_string()))?;
        Ok(ModelResponse::EditImageResponse(
            context.response.get_status().unwrap_or_default(),
            context.response.headers.clone(),
            body,
        ))
    }*/
}
