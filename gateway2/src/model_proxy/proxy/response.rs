use crate::report::STATE;
use aiway_model_protocol::audio::AudioSpeechResponse;
use aiway_model_protocol::chat::ChatCompletionChunkResponse;
use aiway_model_protocol::chat::ChatCompletionResponse;
use aiway_model_protocol::embedding::EmbeddingResponse;
use aiway_model_protocol::image::ImageResponse;
use dashmap::DashMap;
use serde_json::{Value, json};
use std::pin::Pin;
use tokio_stream::Stream;

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

    /// 编辑图像
    EditImageResponse(u16, DashMap<String, String>, ImageResponse),
}

#[derive(thiserror::Error, Debug)]
pub enum ModelError {
    /// 请求模型提供商时发生的错误，如API地址错误，连接失败，服务器无响应等无法连接的情况
    /// 该错误不会进入插件处理
    /// 响应状态码：500
    #[error("{0}")]
    RequestProviderError(String),
    /// 调用模型提供商API时的错误，该错误会进入插件处理
    #[error("{0} {1}")]
    ApiError(u16, String),
    /// SSE流错误，响应`error`事件
    #[error("{0}")]
    StreamError(String),
    /// 不支持的模型错误，响应状态码：400
    #[error("Unsupported model: {0}")]
    UnsupportedModel(String),
    /// 没有可用的提供商，响应状态码：500
    #[error("No available provider")]
    NoAvailableProvider,
    /// 解析错误，响应状态码：500
    #[error("Parse error")]
    Parse(String),
    /// 插件执行错误，响应状态码：500
    #[error("Plugin error: {0}")]
    PluginError(String),
    /// 未知错误，响应状态码：500
    #[error("Unknown error: {0}")]
    Unknown(String),
}
