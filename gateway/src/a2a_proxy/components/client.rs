use aiway_protocol::a2a::Agent;
use anyhow::bail;
use busi::res::Res;
use clap::Parser;
use reqwest::{Client, ClientBuilder};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::LazyLock;
use std::time::Duration;

/// 内部 HTTP 客户端，用于从 Console API 拉取 Agent 列表。
pub struct AgentHttpClient {
    client: Client,
    console: String,
}

pub static AGENT_HTTP_CLIENT: LazyLock<AgentHttpClient> =
    LazyLock::new(|| AgentHttpClient::new(crate::Args::parse().console));

impl AgentHttpClient {
    fn new(console: String) -> Self {
        let client = ClientBuilder::default()
            .connect_timeout(Duration::from_secs(1))
            .build()
            .unwrap();
        Self { client, console }
    }

    async fn fetch_resource<T>(&self, endpoint: &str) -> anyhow::Result<T>
    where
        T: DeserializeOwned + Serialize,
    {
        let response = self.client.get(endpoint).send().await?;
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

    /// 从 Console 拉取 Agent 列表
    pub async fn fetch_agents(&self) -> anyhow::Result<Vec<Agent>> {
        let endpoint = format!("http://{}/api/v1/a2a/agents", self.console);
        self.fetch_resource::<Vec<Agent>>(&endpoint).await
    }
}
