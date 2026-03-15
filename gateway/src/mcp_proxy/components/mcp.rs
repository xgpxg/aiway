use crate::mcp_proxy::components::client::INNER_HTTP_CLIENT;
use crate::mcp_proxy::proxy::MCP_PROXY_POOL;
use aiway_protocol::mcp::mcp::{McpServer, McpServerType, McpTool};
use anyhow::bail;
use dashmap::DashMap;
use logging::log;
use reqwest::Proxy;
use std::collections::HashMap;
use std::process::exit;
use std::sync::OnceLock;
use std::time::Duration;

pub struct McpFactory {
    servers: DashMap<String, McpServer>,
}

pub static MCP_FACTORY: OnceLock<McpFactory> = OnceLock::new();

impl McpFactory {
    pub async fn init() {
        if let Err(e) = Self::load().await {
            log::error!("{}", e);
            exit(1)
        }
    }

    pub async fn load() -> anyhow::Result<()> {
        let mcp_servers = Self::fetch_mcp_servers().await?;
        log::info!("loaded {} mcp servers", mcp_servers.len());
        MCP_FACTORY.get_or_init(|| Self {
            servers: mcp_servers
                .into_iter()
                .map(|mcp| (mcp.name.clone(), mcp))
                .collect::<_>(),
        });

        log::info!(
            "mcp servers: {:?}",
            MCP_FACTORY
                .get()
                .unwrap()
                .servers
                .iter()
                .map(|item| item.key().clone())
                .collect::<Vec<_>>()
        );

        Self::watch();

        Ok(())
    }

    async fn fetch_mcp_servers() -> anyhow::Result<Vec<McpServer>> {
        INNER_HTTP_CLIENT.fetch_mcp().await
    }

    const INTERVAL: Duration = Duration::from_secs(5);

    fn watch() {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Self::INTERVAL);
            loop {
                interval.tick().await;
                let list = Self::fetch_mcp_servers().await;

                let list = match list {
                    Ok(list) => list,
                    Err(e) => {
                        log::error!("fetch mcp server error: {}", e);
                        continue;
                    }
                };

                let old = MCP_FACTORY.get().unwrap();

                let new_servers = list
                    .into_iter()
                    .map(|m| (m.name.clone(), m))
                    .collect::<HashMap<String, McpServer>>();

                // 移除不存在的
                old.servers.retain(|_, item| {
                    if !new_servers.contains_key(&item.name) {
                        log::info!("removed mcp server: {}", item.name);
                        MCP_PROXY_POOL.remove_proxy_client(&item.name);
                        return false;
                    }
                    true
                });

                // 处理新增和变更的
                new_servers.into_iter().for_each(|(name, new_server)| {
                    let need_update = match old.servers.get(&name) {
                        Some(old_server) => old_server.ne(&new_server),
                        None => true,
                    };

                    if need_update {
                        if old.servers.get(&name).is_some() {
                            log::info!("changed mcp server: {}", name);
                            MCP_PROXY_POOL.remove_proxy_client(&name);
                        } else {
                            log::info!("new mcp server enabled: {}", name);
                        }
                        old.servers.insert(name.clone(), new_server);
                    }
                });
            }
        });
    }

    /// 检查server是否存在
    pub fn server_exists(name: &str) -> bool {
        MCP_FACTORY.get().unwrap().servers.contains_key(name)
    }

    /// 获取一个server
    pub async fn get_server(name: &str) -> Option<McpServer> {
        MCP_FACTORY
            .get()
            .unwrap()
            .servers
            .get(name)
            .map(|s| s.clone())
    }

    pub async fn get_tool(name: &str, tool_name: &str) -> Option<McpTool> {
        MCP_FACTORY
            .get()
            .unwrap()
            .servers
            .get(name)
            .map(|s| s.clone())
            .and_then(|s| s.tools.get(tool_name).map(|t| t.clone()))
    }
}
