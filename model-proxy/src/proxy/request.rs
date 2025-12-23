use openai_dive::v1::resources::chat::ChatCompletionParameters;
use openai_dive::v1::resources::embedding::EmbeddingParameters;

/// 对话补全请求
pub type ChatCompletionRequest = ChatCompletionParameters;
/// 嵌入请求
pub type EmbeddingRequest = EmbeddingParameters;
