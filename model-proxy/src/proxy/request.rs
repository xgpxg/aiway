use aiway_model_protocol::audio::AudioSpeechParameters;
use aiway_model_protocol::chat::ChatCompletionParameters;
use aiway_model_protocol::embedding::EmbeddingParameters;
use aiway_model_protocol::image::{CreateImageParameters, EditImageParameters};

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

impl_modify_model_name!(ChatCompletionParameters);
impl_modify_model_name!(EmbeddingParameters);
impl_modify_model_name!(AudioSpeechParameters);
impl ModifyModelName for CreateImageParameters {
    fn get_source_model_name(&self) -> String {
        self.model.clone().expect("model is required")
    }

    fn modify_model_name(mut self, target_model_name: &str) -> Self {
        self.model = Some(target_model_name.to_string());
        self
    }
}
impl ModifyModelName for EditImageParameters {
    fn get_source_model_name(&self) -> String {
        self.model.clone().expect("model is required")
    }

    fn modify_model_name(mut self, target_model_name: &str) -> Self {
        self.model = Some(target_model_name.to_string());
        self
    }
}
