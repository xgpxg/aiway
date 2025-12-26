use crate::components::ModelFactory;
use crate::proxy::proxy::Proxy;
use crate::proxy::request::{AudioSpeechRequest, ChatCompletionRequest};
use crate::proxy::response::{ModelError, ModelResponse};
use rocket::post;
use rocket::serde::json::Json;

/// 对话补全
#[post("/chat/completions", data = "<req>")]
pub async fn chat_completions(
    req: Json<ChatCompletionRequest>,
) -> Result<ModelResponse, ModelError> {
    let req = req.0;
    match ModelFactory::get_provider(&req.model.clone()) {
        Ok(provider) => Proxy::chat_completions(req, &provider).await,
        Err(e) => Err(e),
    }
}

/// 文本转语音
#[post("/audio/speech", data = "<req>")]
pub async fn audio_speech(req: Json<AudioSpeechRequest>) -> Result<ModelResponse, ModelError> {
    let req = req.0;
    match ModelFactory::get_provider(&req.model.clone()) {
        Ok(provider) => Proxy::audio_speech(req, &provider).await,
        Err(e) => Err(e),
    }
}
