use dashmap::DashMap;
use openai_dive::v1::error::APIError;
use openai_dive::v1::resources::audio::AudioSpeechResponse;
use openai_dive::v1::resources::chat::{ChatCompletionChunkResponse, ChatCompletionResponse};
use openai_dive::v1::resources::embedding::EmbeddingResponse;
use openai_dive::v1::resources::image::ImageResponse;
use rocket::Request;
use rocket::futures::{Stream, StreamExt};
use rocket::http::{Header, Status};
use rocket::response::Responder;
use rocket::response::stream::{Event, EventStream};
use serde_json::json;
use std::pin::Pin;

pub enum ModelResponse {
    /// 对话补全（非流式）
    ChatCompletionResponse(u16, DashMap<String, String>, ChatCompletionResponse),
    /// 对话补全（流式）
    ChatCompletionStreamResponse(
        Pin<Box<dyn Stream<Item = Result<ChatCompletionChunkResponse, ModelError>> + Send>>,
    ),
    /// 嵌入
    #[allow(unused)]
    EmbeddingResponse(u16, DashMap<String, String>, EmbeddingResponse),

    /// 语音生成（非流式）
    AudioSpeechResponse(u16, DashMap<String, String>, AudioSpeechResponse),

    /// 创建图像
    CreateImageResponse(u16, DashMap<String, String>, ImageResponse),
}

#[derive(thiserror::Error, Debug)]
pub enum ModelError {
    /// API错误
    /// 调用提供商API时的错误，内部值由openai_dive提供，这里包装一下
    #[error("API error: {0}")]
    ApiError(#[from] APIError),
    /// 不支持的模型错误
    #[error("Unsupported model: {0}")]
    UnsupportedModel(String),
    /// 没有可用的提供商
    #[error("No available provider")]
    NoAvailableProvider,
    /// 解析错误
    #[error("Parse error")]
    Parse(String),
    /// 插件执行错误
    #[error("Plugin error: {0}")]
    PluginError(String),
    /// 未知错误
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl<'r> Responder<'r, 'r> for ModelResponse {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'r> {
        match self {
            ModelResponse::ChatCompletionResponse(status, headers, response) => {
                let mut response = json!(&response).respond_to(request)?;
                response.set_status(Status::new(status));

                for (key, value) in headers {
                    response.set_header(Header::new(key, value));
                }
                Ok(response)
            }
            ModelResponse::ChatCompletionStreamResponse(stream) => {
                let sse_stream = stream.map(move |result| match result {
                    Ok(chunk) => Event::json(&chunk),
                    Err(e) => Event::data(e.to_string()).event("error"),
                });

                let response = EventStream::from(sse_stream).respond_to(request)?;

                Ok(response)
            }
            ModelResponse::EmbeddingResponse(status, headers, response) => {
                let mut response = json!(&response).respond_to(request)?;
                response.set_status(Status::new(status));

                for (key, value) in headers {
                    response.set_header(Header::new(key, value));
                }
                Ok(response)
            }
            ModelResponse::AudioSpeechResponse(status, headers, response) => {
                let mut response = response.bytes.to_vec().respond_to(request)?;
                response.set_status(Status::new(status));

                for (key, value) in headers {
                    response.set_header(Header::new(key, value));
                }

                Ok(response)
            }
            ModelResponse::CreateImageResponse(status, headers, response) => {
                let mut response = json!(&response).respond_to(request)?;
                response.set_status(Status::new(status));

                for (key, value) in headers {
                    response.set_header(Header::new(key, value));
                }
                Ok(response)
            }
        }
    }
}

impl<'r> Responder<'r, 'r> for ModelError {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'r> {
        match self {
            // 调用模型提供商时发生的错误
            Self::ApiError(err) => {
                match err {
                    APIError::AuthenticationError(_) => (Status::Unauthorized, err.to_string()).respond_to(request),
                    APIError::BadRequestError(_) => (Status::BadRequest, err.to_string()).respond_to(request),
                    APIError::PermissionError(_) => (Status::Forbidden, err.to_string()).respond_to(request),
                    APIError::NotFoundError(_) => (Status::NotFound, err.to_string()).respond_to(request),
                    APIError::InvalidRequestError(_) => (Status::ServiceUnavailable, err.to_string()).respond_to(request),
                    APIError::RateLimitError(_) => (Status::TooManyRequests, err.to_string()).respond_to(request),
                    _ => (Status::InternalServerError, err.to_string()).respond_to(request)
                }
            }
            // 不支持的模型错误，返回400
            Self::UnsupportedModel(model) => (
                Status::BadRequest,
                json!({"error": {"code": "400","message": format!("unsupported model: {}", model)}}),
            )
                .respond_to(request),
            Self::NoAvailableProvider => {
                (
                    Status::InternalServerError,
                    json!({"error": {"code": "500","message": "No available provider"}}),
                )
                    .respond_to(request)
            }
            Self::Parse(e) => {
                (
                    Status::InternalServerError,
                    json!({"error": {"code": "500","message": e}}),
                )
                    .respond_to(request)
            }
            ModelError::PluginError(e) => {
                 (
                    Status::InternalServerError,
                    json!({"error": {"code": "500","message": e}}),
                )
                    .respond_to(request)
            }

            // 未知错误，按500返回
            Self::Unknown(message) => (
                Status::InternalServerError,
                json!({"error": {"code": "500","message": message}}),
            )
                .respond_to(request),
        }
    }
}
