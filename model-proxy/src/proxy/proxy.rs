use crate::proxy::request::{AudioSpeechRequest, ChatCompletionRequest, ModifyModelName};
use crate::proxy::response::{ModelError, ModelResponse};
use dashmap::DashMap;
use logging::log;
use openai_dive::v1::api::Client;
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
        context: &HttpContext,
    ) -> Result<ModelResponse, ModelError> {
        let client = get_or_create_client!(req.model, provider);
        let req = Self::modify_model_name(req, provider);

        let mut chat = client.chat();

        if let Some(converter) = &provider.request_converter {
            let plugin_result = PluginFactory::execute(converter, context)
                .await
                .map_err(|e| ModelError::Unknown(e.to_string()))?;

            chat.set_request_converter(Box::new(move |_| plugin_result.clone()));
        }

        if req.stream.unwrap_or(false) {
            let response = chat.create_stream(req).await;
            match response {
                Ok(response) => Ok(ModelResponse::ChatCompletionStreamResponse(response)),
                Err(e) => {
                    log::error!("request model api error: {:?}", e);
                    Err(ModelError::ApiError(e))
                }
            }
        } else {
            let response = chat.create(req).await;
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

        let mut audio = client.audio();

        if let Some(converter) = &provider.request_converter {
            let plugin_result = PluginFactory::execute(converter, context)
                .await
                .map_err(|e| ModelError::Unknown(e.to_string()))?;

            audio.set_request_converter(Box::new(move |_| plugin_result.clone()));
        }

        let response = audio.create_speech(req).await;

        match response {
            Ok(response) => Ok(ModelResponse::AudioSpeechResponse(response)),
            Err(e) => {
                log::error!("request model api error: {:?}", e);
                Err(ModelError::ApiError(e))
            }
        }
    }
}
