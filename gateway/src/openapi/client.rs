use crate::openapi::lb::LoadBalance;
use dashmap::DashMap;
use std::sync::LazyLock;

pub struct HttpClient {
    client: reqwest::Client,
    lb: LoadBalance,
}

pub static HTTP_CLIENT: LazyLock<HttpClient> = LazyLock::new(HttpClient::new);

impl HttpClient {
    pub fn new() -> Self {
        let mut builder = reqwest::ClientBuilder::default();
        builder = builder.connect_timeout(std::time::Duration::from_secs(5));
        // SAFE, Maybe...
        let client = builder.build().unwrap();

        let lb = LoadBalance::new();

        Self { client, lb }
    }

    pub async fn get(
        &self,
        url: &str,
        _headers: DashMap<String, String>,
    ) -> reqwest::Result<reqwest::Response> {
        self.client.get(url).send().await
    }
}
