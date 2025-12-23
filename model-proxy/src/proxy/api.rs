use rocket::post;
use rocket::serde::json::Json;
use crate::components::ModelFactory;
use crate::proxy::proxy::Proxy;
use crate::proxy::request::ChatCompletionRequest;
use crate::proxy::response::{ModelError, ModelResponse};

/// 对话补全
#[post("/chat/completions", data = "<req>")]
pub async fn chat_completions(req: Json<ChatCompletionRequest>) -> Result<ModelResponse, ModelError> {
    let req = req.0;
    match ModelFactory::get_provider(&req.model.clone()) {
        Some(provider) => {
            Proxy::chat_completions(req, &provider).await
        }
        None => Ok(ModelResponse::Empty),
    }
}
