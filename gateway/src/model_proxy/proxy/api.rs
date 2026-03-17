use crate::model_proxy::components::ModelFactory;
use crate::model_proxy::proxy::proxy::Proxy;
use crate::model_proxy::proxy::response::{ModelError, ModelResponse};
use aiway_model_protocol::audio::AudioSpeechParameters;
use aiway_model_protocol::chat::ChatCompletionParameters;
use aiway_model_protocol::image::{CreateImageParameters, EditImageParameters};
use context::HttpContextWrapper;
use rocket::post;

/// 对话补全
#[post("/chat/completions")]
pub async fn chat_completions(context: HttpContextWrapper) -> Result<ModelResponse, ModelError> {
    let body = context.0.request.get_body();
    if body.is_none() || body.as_ref().unwrap().is_empty() {
        return Err(ModelError::Parse("Request body is empty".to_string()));
    }

    let req: ChatCompletionParameters = serde_json::from_slice(body.as_ref().unwrap())
        .map_err(|e| ModelError::Parse(e.to_string()))?;

    match ModelFactory::get_provider(&req.model.clone()) {
        Ok(provider) => Proxy::chat_completions(req, &provider, &context.0).await,
        Err(e) => Err(e),
    }
}

/// 文本转语音
#[post("/audio/speech")]
pub async fn audio_speech(context: HttpContextWrapper) -> Result<ModelResponse, ModelError> {
    let body = context.0.request.get_body();
    if body.is_none() || body.as_ref().unwrap().is_empty() {
        return Err(ModelError::Parse("Request body is empty".to_string()));
    }

    let req: AudioSpeechParameters = serde_json::from_slice(body.as_ref().unwrap())
        .map_err(|e| ModelError::Parse(e.to_string()))?;
    match ModelFactory::get_provider(&req.model.clone()) {
        Ok(provider) => Proxy::audio_speech(req, &provider, &context.0).await,
        Err(e) => Err(e),
    }
}

/// 创建图像
#[post("/images/generations")]
pub async fn images_generations(context: HttpContextWrapper) -> Result<ModelResponse, ModelError> {
    let body = context.0.request.get_body();
    if body.is_none() || body.as_ref().unwrap().is_empty() {
        return Err(ModelError::Parse("Request body is empty".to_string()));
    }

    let req: CreateImageParameters = serde_json::from_slice(body.as_ref().unwrap())
        .map_err(|e| ModelError::Parse(e.to_string()))?;
    match ModelFactory::get_provider(&req.model.clone().unwrap_or_default()) {
        Ok(provider) => Proxy::create_image(req, &provider, &context.0).await,
        Err(e) => Err(e),
    }
}

/// 编辑图像
#[post("/images/edits")]
pub async fn images_edits(context: HttpContextWrapper) -> Result<ModelResponse, ModelError> {
    let body = context.0.request.get_body();
    if body.is_none() || body.as_ref().unwrap().is_empty() {
        return Err(ModelError::Parse("Request body is empty".to_string()));
    }

    let req: EditImageParameters = serde_json::from_slice(body.as_ref().unwrap())
        .map_err(|e| ModelError::Parse(e.to_string()))?;
    match ModelFactory::get_provider(&req.model.clone().unwrap_or_default()) {
        Ok(provider) => Proxy::edit_image(req, &provider, &context.0).await,
        Err(e) => Err(e),
    }
}
