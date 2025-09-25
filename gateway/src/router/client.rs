//! # 网关和控制台的交互
//!
use crate::Args;
use anyhow::bail;
use clap::Parser;
use protocol::common::res::Res;
use protocol::gateway::{Plugin, Route, Service};
use reqwest::{Client, ClientBuilder};
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

    pub async fn fetch_routes(&self) -> anyhow::Result<Vec<Route>> {
        let endpoint = format!("http://{}/api/v1/gateway/routes", self.args.console);
        match self.get(endpoint, HashMap::new()).await {
            Ok(response) => {
                let res = response.json::<Res<Vec<Route>>>().await?;
                if res.is_success() {
                    Ok(res.data.unwrap_or_default())
                } else {
                    bail!("console return error: {}", res.msg);
                }
            }
            Err(e) => {
                bail!("fetch routes error: {}", e);
            }
        }
    }

    pub async fn fetch_services(&self) -> anyhow::Result<Vec<Service>> {
        let endpoint = format!("http://{}/api/v1/gateway/services", self.args.console);
        match self.get(endpoint, HashMap::new()).await {
            Ok(response) => {
                let res = response.json::<Res<Vec<Service>>>().await?;
                if res.is_success() {
                    Ok(res.data.unwrap_or_default())
                } else {
                    bail!("console return error: {}", res.msg);
                }
            }
            Err(e) => {
                bail!("fetch routes error: {}", e);
            }
        }
    }

    pub async fn fetch_plugins(&self) -> anyhow::Result<Vec<Plugin>> {
        let endpoint = format!("http://{}/api/v1/gateway/plugins", self.args.console);
        match self.get(endpoint, HashMap::new()).await {
            Ok(response) => {
                let res = response.json::<Res<Vec<Plugin>>>().await?;
                if res.is_success() {
                    Ok(res.data.unwrap_or_default())
                } else {
                    bail!("console return error: {}", res.msg);
                }
            }
            Err(e) => {
                bail!("fetch routes error: {}", e);
            }
        }
    }
}
