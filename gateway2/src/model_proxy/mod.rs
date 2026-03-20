use crate::Args;
use bytes::Bytes;
use reqwest::Response;

mod components;
mod proxy;

use crate::model_proxy::proxy::ModelError;
use crate::model_proxy::proxy::api::chat_completions;
pub use components::ModelFactory;

pub async fn model_endpoint(path: &str, body: Bytes) -> Result<Response, ModelError> {
    match path {
        "/v1/model/chat/completions" => chat_completions(Some(body)).await,
        _ => {
            unimplemented!()
        }
    }
}
pub async fn init(args: &Args) {
    // 初始化插件管理器
    plugin_manager::init(&args.console).await;

    // 初始化模型
    components::ModelFactory::init().await;
}
