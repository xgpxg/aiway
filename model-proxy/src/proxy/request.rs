use openai_dive::v1::resources::audio::AudioSpeechParameters;
use openai_dive::v1::resources::chat::ChatCompletionParameters;
use openai_dive::v1::resources::embedding::EmbeddingParameters;

/// 对话补全请求
pub type ChatCompletionRequest = ChatCompletionParameters;
/// 嵌入请求
#[allow(unused)]
pub type EmbeddingRequest = EmbeddingParameters;

pub type AudioSpeechRequest = AudioSpeechParameters;

/// 修改模型名称
///
/// 用于将请求的模型名称映射为提供商的真实模型名称，解决同一模型在不同提供商下的命名不一致问题
pub trait ModifyModelName {
    /// 获取源模型名称，即请求中的
    fn get_source_model_name(&self) -> String;
    /// 修改模型名称
    /// - `target_model_name`: 提供商处的对应的真实模型名
    fn modify_model_name(self, target_model_name: &str) -> Self;
}

macro_rules! impl_modify_model_name {
    ($type:ty) => {
        impl ModifyModelName for $type {
            fn get_source_model_name(&self) -> String {
                self.model.clone()
            }

            fn modify_model_name(mut self, target_model_name: &str) -> Self {
                self.model = target_model_name.to_string();
                self
            }
        }
    };
}

impl_modify_model_name!(ChatCompletionRequest);
impl_modify_model_name!(EmbeddingRequest);
impl_modify_model_name!(AudioSpeechRequest);