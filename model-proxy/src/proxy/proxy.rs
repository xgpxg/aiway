use crate::proxy::client;
use crate::proxy::client::Client;
use crate::proxy::request::{
    AudioSpeechRequest, ChatCompletionRequest, CreateImageRequest, ModifyModelName,
};
use crate::proxy::response::{ModelError, ModelResponse};
use dashmap::DashMap;
use logging::log;
use openai_dive::v1::resources::audio::{AudioSpeechParameters, AudioSpeechResponse};
use openai_dive::v1::resources::chat::{ChatCompletionChunkResponse, ChatCompletionResponse};
use plugin_manager::PluginFactory;
use protocol::gateway::HttpContext;
use protocol::model::Provider;
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

/// 调用插件转换请求参数
macro_rules! convert_request {
    ($req:expr, $provider:expr, $context:expr) => {
        if let Some(converter) = &$provider.request_converter {
            PluginFactory::execute(converter, $context)
                .await
                .map_err(|e| ModelError::Unknown(e.to_string()))?
        } else {
            serde_json::to_value($req).map_err(|e| ModelError::Unknown(e.to_string()))?
        }
    };
}
impl Proxy {
    /// 移除某个模型下的所有Client实例
    ///
    /// 仅当模型配置发生变更（新增、修改、删除、提供商变更）时才需要调用此方法
    pub fn remove_clients(model_name: &str) {
        let model_name = model_name.to_string();
        PROXY.clients.retain(|(model, _), _| *model != model_name);
    }

    pub fn modify_model_name<R: ModifyModelName>(req: R, provider: &Provider) -> R {
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
        req: ChatCompletionRequest,
        provider: &Provider,
        context: &HttpContext,
    ) -> Result<ModelResponse, ModelError> {
        let client = get_or_create_client!(req.model, provider);
        let req = Self::modify_model_name(req, provider);
        let converted_req = convert_request!(&req, provider, context);

        if req.stream.unwrap_or(false) {
            let response = client
                .post_stream(&provider.api_url, &converted_req, None)
                .await;
            Ok(ModelResponse::ChatCompletionStreamResponse(response))
        } else {
            let response = client.post(&provider.api_url, &converted_req, None).await;
            match response {
                Ok(response) => Ok(ModelResponse::ChatCompletionResponse(response)),
                Err(e) => {
                    log::error!("request model api error: {:?}", e);
                    Err(ModelError::ApiError(e))
                }
            }
        }
    }

    /// 文本转语音
    pub async fn audio_speech(
        req: AudioSpeechRequest,
        provider: &Provider,
        context: &HttpContext,
    ) -> Result<ModelResponse, ModelError> {
        let client = get_or_create_client!(req.model, provider);
        let req = Self::modify_model_name(req, provider);
        let req = convert_request!(req, provider, context);

        let response = client.post_raw(&provider.api_url, &req, None).await;

        match response {
            Ok(response) => Ok(ModelResponse::AudioSpeechResponse(AudioSpeechResponse {
                bytes: response,
            })),
            Err(e) => {
                log::error!("request model api error: {:?}", e);
                Err(ModelError::ApiError(e))
            }
        }
    }

    /// 创建图像(文生图)
    pub async fn create_image(
        req: CreateImageRequest,
        provider: &Provider,
        context: &HttpContext,
    ) -> Result<ModelResponse, ModelError> {
        context.request.insert_state("provider", provider.clone());
        let client = get_or_create_client!(req.model.clone().unwrap_or_default(), provider);
        let req = Self::modify_model_name(req, provider);
        let req = convert_request!(req, provider, context);
        let response = client.post(&provider.api_url, &req, None).await;

        match response {
            Ok(response) => Ok(ModelResponse::CreateImageResponse(response)),
            Err(e) => {
                log::error!("request model api error: {:?}", e);
                Err(ModelError::ApiError(e))
            }
        }
    }
}
