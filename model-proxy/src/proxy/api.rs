use crate::components::ModelFactory;
use crate::proxy::proxy::Proxy;
use crate::proxy::request::{AudioSpeechRequest, ChatCompletionRequest, CreateImageRequest};
use crate::proxy::response::{ModelError, ModelResponse};
use context::HttpContextOnce;
use rocket::post;
use rocket::serde::json::Json;

/// 对话补全
#[post("/chat/completions", data = "<req>")]
pub async fn chat_completions(
    req: Json<ChatCompletionRequest>,
    context: HttpContextOnce,
) -> Result<ModelResponse, ModelError> {
    match ModelFactory::get_provider(&req.model.clone()) {
        Ok(provider) => Proxy::chat_completions(req.0, &provider, &context.0).await,
        Err(e) => Err(e),
    }
}

/// 文本转语音
#[post("/audio/speech", data = "<req>")]
pub async fn audio_speech(
    req: Json<AudioSpeechRequest>,
    context: HttpContextOnce,
) -> Result<ModelResponse, ModelError> {
    match ModelFactory::get_provider(&req.model.clone()) {
        Ok(provider) => Proxy::audio_speech(req.0, &provider, &context.0).await,
        Err(e) => Err(e),
    }
}

#[post("/images/generations", data = "<req>")]
pub async fn images_generations(
    req: Json<CreateImageRequest>,
    context: HttpContextOnce,
) -> Result<ModelResponse, ModelError> {
    match ModelFactory::get_provider(&req.model.clone().unwrap_or_default()) {
        Ok(provider) => Proxy::create_image(req.0, &provider, &context.0).await,
        Err(e) => Err(e),
    }
}
