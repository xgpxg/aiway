use crate::proxy::request::ChatCompletionRequest;
use crate::proxy::response::{ModelError, ModelResponse};
use dashmap::DashMap;
use logging::log;
use openai_dive::v1::api::Client;
use protocol::model::Provider;
use std::sync::LazyLock;

pub struct Proxy {
    clients: DashMap<String, Client>,
}

static PROXY: LazyLock<Proxy> = LazyLock::new(|| Proxy {
    clients: DashMap::new(),
});

impl Proxy {
    pub async fn chat_completions(
        req: ChatCompletionRequest,
        provider: &Provider,
    ) -> Result<ModelResponse, ModelError> {
        let client = PROXY.clients.entry(provider.name.clone()).or_insert_with(|| {
            log::info!("Creating client for provider {}", provider.name);
            let mut client = Client::new(provider.api_key.clone().unwrap_or_default());
            client.set_base_url(&provider.api_url);
            client
        });
        if req.stream.unwrap_or(false) {
            let response = client.chat().create_stream(req).await;
            match response {
                Ok(response) => Ok(ModelResponse::ChatCompletionStreamResponse(response)),
                Err(e) => Err(ModelError::ApiError(e)),
            }
        } else {
            let response = client.chat().create(req).await;
            match response {
                Ok(response) => Ok(ModelResponse::ChatCompletionResponse(response)),
                Err(e) => Err(ModelError::ApiError(e)),
            }
        }
    }
}
