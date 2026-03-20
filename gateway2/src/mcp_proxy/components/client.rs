use crate::Args;
use aiway_protocol::mcp::mcp::McpServer;
use anyhow::bail;
use busi::res::Res;
use clap::Parser;
use reqwest::{Client, ClientBuilder};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::LazyLock;

pub struct InnerHttpClient {
    client: Client,
    args: Args,
}

pub static INNER_HTTP_CLIENT: LazyLock<InnerHttpClient> = LazyLock::new(InnerHttpClient::new);

impl InnerHttpClient {
    pub fn new() -> Self {
        let client = ClientBuilder::default()
            .connect_timeout(std::time::Duration::from_secs(1))
            .build()
            .unwrap();
        let args = Args::parse();
        Self { client, args }
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

    pub async fn fetch_mcp(&self) -> anyhow::Result<Vec<McpServer>> {
        let endpoint = format!("http://{}/api/v1/mcp/servers", self.args.console);
        let servers = self.fetch_resource::<Vec<McpServer>>(endpoint).await?;
        Ok(servers)
    }
}
