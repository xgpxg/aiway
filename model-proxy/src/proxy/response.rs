use openai_dive::v1::error::APIError;
use openai_dive::v1::resources::chat::{ChatCompletionChunkResponse, ChatCompletionResponse};
use openai_dive::v1::resources::embedding::EmbeddingResponse;
use rocket::Request;
use rocket::futures::{Stream, StreamExt};
use rocket::response::Responder;
use rocket::response::stream::TextStream;
use serde_json::json;
use std::pin::Pin;

pub enum ModelResponse {
    #[allow(unused)]
    Empty,
    /// 对话补全（非流式）
    ChatCompletionResponse(ChatCompletionResponse),
    /// 对话补全（流式）
    ChatCompletionStreamResponse(
        Pin<Box<dyn Stream<Item = Result<ChatCompletionChunkResponse, APIError>> + Send>>,
    ),
    /// 嵌入
    #[allow(unused)]
    EmbeddingResponse(EmbeddingResponse),
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
    /// 未知错误
    #[allow(unused)]
    #[error("Unknown error: {0}")]
    Unknown(String),
}

enum SseEvent {
    Data(String),
    Error(String),
    Done,
}
impl SseEvent {
    fn to_string(&self) -> String {
        match self {
            SseEvent::Data(data) => format!("data: {}\n\n", data),
            SseEvent::Error(error) => {
                format!("event: error\ndata: {}\n\n", error)
            }
            SseEvent::Done => "data: [DONE]\n\n".to_string(),
        }
    }
}

impl<'r> Responder<'r, 'r> for ModelResponse {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'r> {
        match self {
            ModelResponse::Empty => ().respond_to(request),
            ModelResponse::ChatCompletionResponse(response) => json!(&response).respond_to(request),
            ModelResponse::ChatCompletionStreamResponse(stream) => {
                let sse_stream = stream
                    .map(move |result| match result {
                        Ok(chunk) => {
                            SseEvent::Data(serde_json::to_string(&chunk).unwrap_or_default())
                                .to_string()
                        }
                        Err(e) => SseEvent::Error(e.to_string()).to_string(),
                    })
                    .chain(rocket::futures::stream::once(async {
                        SseEvent::Done.to_string()
                    }));

                let mut response = TextStream::from(sse_stream).respond_to(request)?;

                response.set_header(rocket::http::ContentType::new("text", "event-stream"));
                response.set_header(rocket::http::Header::new("Cache-Control", "no-cache"));
                response.set_header(rocket::http::Header::new("Connection", "keep-alive"));
                response.set_header(rocket::http::Header::new("X-Accel-Buffering", "no"));

                Ok(response)
            }
            ModelResponse::EmbeddingResponse(response) => json!(&response).respond_to(request),
        }
    }
}

impl<'r> Responder<'r, 'r> for ModelError {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'r> {
        match self {
            // 调用模型提供商时发生的错误
            Self::ApiError(err) => {
                match err {
                    APIError::AuthenticationError(_) => (rocket::http::Status::Unauthorized, err.to_string()).respond_to(request),
                    APIError::BadRequestError(_) => (rocket::http::Status::BadRequest, err.to_string()).respond_to(request),
                    APIError::PermissionError(_) => (rocket::http::Status::Forbidden, err.to_string()).respond_to(request),
                    APIError::NotFoundError(_) => (rocket::http::Status::NotFound, err.to_string()).respond_to(request),
                    APIError::InvalidRequestError(_) => (rocket::http::Status::ServiceUnavailable, err.to_string()).respond_to(request),
                    APIError::RateLimitError(_) => (rocket::http::Status::TooManyRequests, err.to_string()).respond_to(request),
                    _ => (rocket::http::Status::InternalServerError, err.to_string()).respond_to(request)
                }
            }
            // 不支持的模型错误，返回400
            Self::UnsupportedModel(model) => (
                rocket::http::Status::BadRequest,
                json!({"error": {"code": "400","message": format!("unsupported model: {}", model)}}),
            )
                .respond_to(request),
            // 未知错误，按500返回
            Self::Unknown(message) => (
                rocket::http::Status::InternalServerError,
                json!({"error": {"code": "500","message": message}}),
            )
                .respond_to(request),

            Self::NoAvailableProvider => {
                (
                    rocket::http::Status::InternalServerError,
                    json!({"error": {"code": "500","message": "No available provider"}}),
                )
                    .respond_to(request)
            }
        }
    }
}
