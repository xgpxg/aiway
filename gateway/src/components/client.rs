//! # 网关和控制台的交互
//!
use crate::Args;
use busi::res::Res;
use aiway_protocol::gateway::{Config, Firewall, GlobalFilter, Plugin, Route, Service};
use anyhow::bail;
use clap::Parser;
use reqwest::{Client, ClientBuilder};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::path::PathBuf;
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

    pub async fn fetch_routes(&self) -> anyhow::Result<Vec<Route>> {
        let endpoint = format!("http://{}/api/v1/gateway/routes", self.args.console);
        let mut routes = self.fetch_resource::<Vec<Route>>(endpoint).await?;
        routes.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(routes)
    }

    pub async fn fetch_services(&self) -> anyhow::Result<Vec<Service>> {
        let endpoint = format!("http://{}/api/v1/gateway/services", self.args.console);
        let mut services = self.fetch_resource::<Vec<Service>>(endpoint).await?;
        services.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(services)
    }

    pub async fn fetch_plugins(&self) -> anyhow::Result<Vec<Plugin>> {
        let endpoint = format!("http://{}/api/v1/gateway/plugins", self.args.console);
        let mut plugins = self.fetch_resource::<Vec<Plugin>>(endpoint).await?;
        plugins.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(plugins)
    }

    pub async fn fetch_global_filter(&self) -> anyhow::Result<GlobalFilter> {
        let endpoint = format!("http://{}/api/v1/gateway/global/filter", self.args.console);
        let global_filter = self.fetch_resource::<GlobalFilter>(endpoint).await?;
        Ok(global_filter)
    }

    pub async fn fetch_firewall(&self) -> anyhow::Result<Firewall> {
        let endpoint = format!("http://{}/api/v1/gateway/firewall", self.args.console);
        let firewall = self.fetch_resource::<Firewall>(endpoint).await?;
        Ok(firewall)
    }

    pub async fn fetch_ip_region_file(&self) -> anyhow::Result<PathBuf> {
        let endpoint = format!(
            "http://{}/api/v1/gateway/download-ip-region-file",
            self.args.console
        );
        match self.get(endpoint, HashMap::new()).await {
            Ok(response) => {
                if let Err(e) = response.error_for_status_ref() {
                    bail!("fetch ip region file error: {}", e);
                }
                let res = response.bytes().await?;

                let temp_dir = std::env::temp_dir();
                let path = temp_dir.join("ip2region_v4.xdb");
                std::fs::write(&path, res)?;
                Ok(path)
            }
            Err(e) => {
                bail!("fetch ip region file error: {}", e);
            }
        }
    }

    pub async fn fetch_config(&self) -> anyhow::Result<Config> {
        let endpoint = format!("http://{}/api/v1/gateway/config", self.args.console);
        let config = self.fetch_resource::<Config>(endpoint).await?;
        Ok(config)
    }
}
