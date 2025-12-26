use crate::proxy::request::{AudioSpeechRequest, ChatCompletionRequest, ModifyModelName};
use crate::proxy::response::{ModelError, ModelResponse};
use dashmap::DashMap;
use logging::log;
use openai_dive::v1::api::Client;
use protocol::model::Provider;
use std::sync::LazyLock;
use rocket::serde::json::Json;

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
                let mut client = Client::new($provider.api_key.clone().unwrap_or_default());
                client.set_base_url(&$provider.api_url);
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
    ) -> Result<ModelResponse, ModelError> {
        let client = get_or_create_client!(req.model, provider);
        let req = Self::modify_model_name(req, provider);
        if req.stream.unwrap_or(false) {
            let response = client.chat().create_stream(req).await;
            match response {
                Ok(response) => Ok(ModelResponse::ChatCompletionStreamResponse(response)),
                Err(e) => {
                    log::error!("request model api error: {:?}", e);
                    Err(ModelError::ApiError(e))
                }
            }
        } else {
            let response = client.chat().create(req).await;
            match response {
                Ok(response) => Ok(ModelResponse::ChatCompletionResponse(response)),
                Err(e) => {
                    log::error!("request model api error: {:?}", e);
                    Err(ModelError::ApiError(e))
                }
            }
        }
    }

    pub async fn audio_speech(req: AudioSpeechRequest, provider: &Provider) -> Result<ModelResponse, ModelError> {
        let client = get_or_create_client!(req.model, provider);
        let req = Self::modify_model_name(req, provider);
        let response = client.audio().create_speech(req).await;
        match response {
            Ok(response) => Ok(ModelResponse::AudioSpeechResponse(response)),
            Err(e) => {
                log::error!("request model api error: {:?}", e);
                Err(ModelError::ApiError(e))
            }
        }
    }
}
