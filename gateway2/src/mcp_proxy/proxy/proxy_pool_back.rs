use crate::mcp_proxy::components::McpFactory;
use crate::mcp_proxy::proxy::handler::HTTP_CLIENT;
use crate::mcp_proxy::proxy::response::McpRes;
use aiway_protocol::mcp::mcp::McpServer;
use aiway_protocol::rmcp::model::{JsonRpcMessage, JsonRpcRequest, RequestId, ServerResult};
use anyhow::bail;
use dashmap::DashMap;
use http::{HeaderMap, HeaderName, HeaderValue, Method};
use serde_json::json;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::atomic::{AtomicI64, AtomicU64};
use std::sync::{Arc, LazyLock};

#[derive(Debug)]
pub struct McpProxyConnection {
    url: String,
    headers: HashMap<String, String>,
    mcp_session_id: String,
    id: AtomicI64,
}

impl McpProxyConnection {
    pub fn new(url: &str, headers: HashMap<String, String>) -> Self {
        McpProxyConnection {
            url: url.to_string(),
            headers,
            mcp_session_id: Default::default(),
            id: AtomicI64::default(),
        }
    }
    async fn initialize(&mut self) -> anyhow::Result<()> {
        let id = self.id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let headers = HeaderMap::from_iter(self.headers.iter().map(|(k, v)| {
            let name = HeaderName::from_str(k).expect("Invalid header name");
            let value = HeaderValue::from_str(v).expect("Invalid header value");
            (name, value)
        }));
        let res = HTTP_CLIENT
            .client
            .request(Method::POST, &self.url)
            .header("accept", "application/json, text/event-stream")
            .headers(headers)
            .json(&json!({"jsonrpc":"2.0","id":id,"method":"initialize","params":{"capabilities":{},"clientInfo":{"name":"rmcp","version":"1.2.0"},"protocolVersion":"2025-06-18"}}))
            .send()
            .await?;

        let mcp_session_id = res
            .headers()
            .get("mcp-session-id")
            .map(|h| h.to_str().unwrap().to_string())
            .unwrap_or_default();

        self.mcp_session_id = mcp_session_id;

        let _ = HTTP_CLIENT
            .client
            .request(Method::POST, &self.url)
            .header("accept", "application/json, text/event-stream")
            .header("mcp-session-id", &self.mcp_session_id)
            .json(&json!({"jsonrpc":"2.0","method":"notifications/initialized","params":{}}))
            .send()
            .await?;

        Ok(())
    }

    pub async fn request(&self, message: JsonRpcRequest) -> anyhow::Result<McpRes<ServerResult>> {
        let headers = HeaderMap::from_iter(self.headers.iter().map(|(k, v)| {
            let name = HeaderName::from_str(k).expect("Invalid header name");
            let value = HeaderValue::from_str(v).expect("Invalid header value");
            (name, value)
        }));
        let res = HTTP_CLIENT
            .client
            .request(Method::POST, &self.url)
            .header("accept", "application/json, text/event-stream")
            .header("mcp-session-id", &self.mcp_session_id)
            .headers(headers)
            .json(&message)
            .send()
            .await?;

        let res = res.text().await?;

        let res: McpRes<ServerResult> = serde_json::from_str(&res)?;

        Ok(res)
    }
}

#[derive(Debug)]
pub struct McpProxyPool {
    connections: DashMap<String, Arc<McpProxyConnection>>,
}
impl McpProxyPool {
    pub fn new() -> Self {
        McpProxyPool {
            connections: DashMap::new(),
        }
    }
    pub async fn get_connection(
        &self,
        server_name: &str,
    ) -> anyhow::Result<Arc<McpProxyConnection>> {
        if let Some(connection) = self.connections.get(server_name) {
            log::info!("Get connection from pool: {}", server_name);
            return Ok(connection.clone());
        }

        log::info!("Create new connection: {}", server_name);

        let server = McpFactory::get_server(server_name).await;
        if server.is_none() {
            bail!("Mcp server not found");
        }
        let server = server.unwrap();
        let proxy_config = server.proxy_config;
        if proxy_config.is_none() {
            bail!("Mcp server proxy config not found");
        }
        let proxy_config = proxy_config.unwrap();
        let url = proxy_config.url;
        let headers = proxy_config.headers;

        let mut connection = McpProxyConnection::new(&url, headers);
        connection.initialize().await?;
        let connection = Arc::new(connection);
        self.connections
            .insert(server_name.to_string(), connection.clone());
        Ok(connection)
    }
}

pub static MCP_PROXY_POOL: LazyLock<McpProxyPool> = LazyLock::new(McpProxyPool::new);
