use crate::model_proxy::components::ModelFactory;
use crate::model_proxy::proxy::proxy::Proxy;
use crate::model_proxy::proxy::response::{ModelError, ModelResponse};
use context::{HttpContextOnce, HttpContextWrapper};
use rocket::post;
use rocket::serde::json::Json;
use aiway_model_protocol::audio::AudioSpeechParameters;
use aiway_model_protocol::chat::ChatCompletionParameters;
use aiway_model_protocol::image::{CreateImageParameters, EditImageParameters};

/// 对话补全
#[post("/chat/completions")]
pub async fn chat_completions(
    context: HttpContextWrapper,
) -> Result<ModelResponse, ModelError> {
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
#[post("/audio/speech", data = "<req>")]
pub async fn audio_speech(
    req: Json<AudioSpeechParameters>,
    context: HttpContextOnce,
) -> Result<ModelResponse, ModelError> {
    match ModelFactory::get_provider(&req.model.clone()) {
        Ok(provider) => Proxy::audio_speech(req.0, &provider, &context.0).await,
        Err(e) => Err(e),
    }
}

/// 创建图像
#[post("/images/generations", data = "<req>")]
pub async fn images_generations(
    req: Json<CreateImageParameters>,
    context: HttpContextOnce,
) -> Result<ModelResponse, ModelError> {
    match ModelFactory::get_provider(&req.model.clone().unwrap_or_default()) {
        Ok(provider) => Proxy::create_image(req.0, &provider, &context.0).await,
        Err(e) => Err(e),
    }
}

/// 编辑图像
#[post("/images/edits", data = "<req>")]
pub async fn images_edits(
    req: Json<EditImageParameters>,
    context: HttpContextOnce,
) -> Result<ModelResponse, ModelError> {
    match ModelFactory::get_provider(&req.model.clone().unwrap_or_default()) {
        Ok(provider) => Proxy::edit_image(req.0, &provider, &context.0).await,
        Err(e) => Err(e),
    }
}
