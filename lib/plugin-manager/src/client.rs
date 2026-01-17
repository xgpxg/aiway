//! # 网关和控制台的交互
//!
use crate::CONSOLE;
use anyhow::bail;
use busi::res::Res;
use aiway_protocol::gateway::Plugin;
use reqwest::{Client, ClientBuilder};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::sync::LazyLock;

pub struct InnerHttpClient {
    client: Client,
}

pub static INNER_HTTP_CLIENT: LazyLock<InnerHttpClient> = LazyLock::new(InnerHttpClient::new);

impl InnerHttpClient {
    pub fn new() -> Self {
        let client = ClientBuilder::default()
            .connect_timeout(std::time::Duration::from_secs(1))
            .build()
            .unwrap();
        Self { client }
    }
}

impl InnerHttpClient {
    async fn get(
        &self,
        url: impl Into<String>,
        query: HashMap<String, String>,
    ) -> reqwest::Result<reqwest::Response> {
        self.client
            .get(url.into().as_str())
            .query(&query)
            .send()
            .await
    }

    async fn fetch_resource<T>(&self, endpoint: String) -> anyhow::Result<T>
    where
        T: DeserializeOwned + Serialize,
    {
        match self.get(endpoint, HashMap::new()).await {
            Ok(response) => {
                if let Err(e) = response.error_for_status_ref() {
                    bail!("http error: {}", e);
                }
                let res = response.json::<Res<T>>().await?;
                if res.is_success() {
                    res.data.ok_or_else(|| anyhow::anyhow!("no data returned"))
                } else {
                    bail!("console returned error: {}", res.msg);
                }
            }
            Err(e) => bail!("network error: {}", e),
        }
    }

    pub async fn fetch_plugins(&self) -> anyhow::Result<Vec<Plugin>> {
        let endpoint = format!("http://{}/api/v1/gateway/plugins", CONSOLE.get().unwrap());
        let mut plugins = self.fetch_resource::<Vec<Plugin>>(endpoint).await?;
        plugins.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(plugins)
    }
}
