use crate::components::ModelFactory;
use crate::proxy::HttpContextWrapper;
use crate::proxy::proxy::Proxy;
use crate::proxy::request::{AudioSpeechRequest, ChatCompletionRequest};
use crate::proxy::response::{ModelError, ModelResponse};
use protocol::gateway::HttpContext;
use rocket::serde::json::Json;
use rocket::{Request, State, post};

/// 对话补全
#[post("/chat/completions", data = "<req>")]
pub async fn chat_completions(
    req: Json<ChatCompletionRequest>,
    context: HttpContextWrapper,
) -> Result<ModelResponse, ModelError> {
    let req = req.0;
    context
        .0
        .request
        .set_body(serde_json::to_vec(&req).unwrap());
    match ModelFactory::get_provider(&req.model.clone()) {
        Ok(provider) => Proxy::chat_completions(req, &provider, &context.0).await,
        Err(e) => Err(e),
    }
}

/// 文本转语音
#[post("/audio/speech", data = "<req>")]
pub async fn audio_speech(
    req: Json<AudioSpeechRequest>,
    context: HttpContextWrapper,
) -> Result<ModelResponse, ModelError> {
    let req = req.0;
    context
        .0
        .request
        .set_body(serde_json::to_vec(&req).unwrap());
    match ModelFactory::get_provider(&req.model.clone()) {
        Ok(provider) => Proxy::audio_speech(req, &provider, &context.0).await,
        Err(e) => Err(e),
    }
}
