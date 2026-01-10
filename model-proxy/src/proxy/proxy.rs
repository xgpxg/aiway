//! # 模型代理
//! 需要尽量兼容OpenAI格式，部分场景可适当扩展
//!
//! 整体执行流程：
//! 网关 → model-proxy → 获取提供商 → 模型名称映射 → 请求参数转换 → 调用提供商 → 响应参数转换 → 返回结果
//!
use crate::proxy::client::Client;
use crate::proxy::request::{
    AudioSpeechRequest, ChatCompletionRequest, CreateImageRequest, ModifyModelName,
};
use crate::proxy::response::{ModelError, ModelResponse};
use dashmap::DashMap;
use logging::log;
use openai_dive::v1::resources::audio::AudioSpeechResponse;
use openai_dive::v1::resources::chat::ChatCompletionChunkResponse;
use plugin_manager::PluginFactory;
use protocol::common::constants::BAN_HEADERS;
use protocol::gateway::HttpContext;
use protocol::model::Provider;
use reqwest::Response;
use rocket::serde::Serialize;
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

    async fn convert_request<R: Serialize>(
        request_body: R,
        provider: &Provider,
        context: &HttpContext,
    ) -> Result<(), ModelError> {
        context.request.set_body(
            serde_json::to_vec(&request_body)
                .map_err(|e| ModelError::Parse(e.to_string()))?
                .into(),
        );
        context.request.insert_state("provider", provider.clone());
        if let Some(converter) = &provider.request_converter {
            // 调用插件执行转换，在插件内部更新context的body
            PluginFactory::execute(converter, context)
                .await
                .map_err(|e| ModelError::PluginError(e.to_string()))?;
        }
        Ok(())
    }

    async fn convert_response(
        response: Response,
        provider: &Provider,
        context: &HttpContext,
    ) -> Result<(), ModelError> {
        println!("response: {:?}", response);
        context.response.set_status(response.status().as_u16());
        context.response.set_headers(
            response
                .headers()
                .iter()
                .filter(|(h, _)| !BAN_HEADERS.contains(h.as_str()))
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string())),
        );
        context
            .response
            .set_body(response.bytes().await.unwrap_or_default());

        // 调用插件执行转换，在插件内部更新context的body
        // 插件内部需要处理成功和失败的情况
        if let Some(converter) = &provider.response_converter {
            PluginFactory::execute(converter, context)
                .await
                .map_err(|e| ModelError::PluginError(e.to_string()))?;
        }

        // 插件执行完成后（或者没有插件执行）响应状态码非200-299，则直接返回错误
        if !context.response.is_success() {
            return Err(ModelError::ApiError(
                context.response.get_status().unwrap_or_default(),
                String::from_utf8_lossy(&context.response.body.take().unwrap_or_default())
                    .to_string(),
            ));
        }

        Ok(())
    }

    /// 对话补全
    pub async fn chat_completions(
        req: ChatCompletionRequest,
        provider: &Provider,
        context: &HttpContext,
    ) -> Result<ModelResponse, ModelError> {
        let client = get_or_create_client!(req.model, provider);
        let req = Self::modify_model_name(req, provider);
        Self::convert_request(&req, provider, context).await?;

        let request_body = context.request.get_body().cloned().unwrap_or_default();

        if req.stream.unwrap_or(false) {
            // 通常情况下，模型提供商的对话补全接口都兼容OpenAI格式，无需转换
            // 所以这里先判断下是否有响应转换器，没有的话，直接返回
            let response_converter = if let Some(response_converter) = &provider.response_converter
            {
                response_converter
            } else {
                // 请求提供商
                let response = client
                    .post_stream(&provider.api_url, request_body, None)
                    .await;
                // 转换错误类型：ApiError -> ModelError
                //let stream = response.map(|item| item.map_err(ModelError::ApiError));

                return Ok(ModelResponse::ChatCompletionStreamResponse(Box::pin(
                    response,
                )));
            };
            // 请求提供商，以Value格式返回
            let response = client
                .post_stream::<_, Value, _>(&provider.api_url, request_body, None)
                .await;
            // 转为context的stream_body支持的stream
            let stream = response.map(|item| {
                item.map_err(|e| {
                    log::error!("Stream item error: {:?}", e);
                    e.into()
                })
                .and_then(|val| {
                    serde_json::to_vec(&val).map_err(|e| {
                        log::error!("Serialization error: {}", e);
                        e.into()
                    })
                })
            });
            // 设置流式的body
            context.response.set_stream_body(Box::pin(stream));

            // 调用插件转换响应结果
            // 该插件应该对stream_body进行处理而不是body
            PluginFactory::execute(response_converter, context)
                .await
                .map_err(|e| ModelError::PluginError(e.to_string()))?;

            // 获取转换后的stream
            let stream = context.response.take_stream_body();

            // 理论上不会出现这种情况，除非在插件中未设置stream_body
            if stream.is_none() {
                return Err(ModelError::Unknown("stream is none".to_string()));
            }

            // 转为ChatCompletionChunkResponse
            let stream = match stream {
                Some(stream) => stream.map(|item| match item {
                    Ok(item) => serde_json::from_slice::<ChatCompletionChunkResponse>(&item)
                        .map_err(|e| {
                            log::error!("Deserialization error: {}", e);
                            ModelError::Parse(e.to_string())
                        }),
                    Err(e) => {
                        log::error!("Stream item error: {:?}", e);
                        Err(ModelError::Parse(e.to_string()))
                    }
                }),
                None => return Err(ModelError::Unknown("stream is none".to_string())),
            };

            Ok(ModelResponse::ChatCompletionStreamResponse(Box::pin(
                stream,
            )))
        } else {
            // 非流式
            let response = client.post(&provider.api_url, request_body, None).await?;
            Self::convert_response(response, provider, context).await?;
            let body = context.response.body.take().unwrap_or_default();
            let body =
                serde_json::from_slice(&body).map_err(|e| ModelError::Parse(e.to_string()))?;
            Ok(ModelResponse::ChatCompletionResponse(
                context.response.get_status().unwrap_or_default(),
                context.response.headers.clone(),
                body,
            ))
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
        req: CreateImageRequest,
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
}
