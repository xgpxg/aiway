use crate::proxy::request::ChatCompletionRequest;
use crate::proxy::response::{ModelError, ModelResponse};
use dashmap::DashMap;
use logging::log;
use openai_dive::v1::api::Client;
use protocol::model::Provider;
use std::sync::LazyLock;

pub struct Proxy {
    clients: DashMap<(String, String), Client>,
}

static PROXY: LazyLock<Proxy> = LazyLock::new(|| Proxy {
    clients: DashMap::new(),
});

impl Proxy {
    pub fn remove_client(model_name: &str, provider_name: &str) {
        PROXY
            .clients
            .remove(&(model_name.to_string(), provider_name.to_string()));
    }
    pub fn remove_clients_by_model(model_name: &str) {
        let model_name = model_name.to_string();
        PROXY.clients.retain(|(model, _), _| *model != model_name);
    }
    pub async fn chat_completions(
        req: ChatCompletionRequest,
        provider: &Provider,
    ) -> Result<ModelResponse, ModelError> {
        let client = PROXY
            .clients
            .entry((req.model.clone(), provider.name.clone()))
            .or_insert_with(|| {
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
