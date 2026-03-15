use crate::mcp_proxy::components::McpFactory;
use crate::mcp_proxy::proxy::response::McpRes;
use aiway_protocol::mcp::mcp::McpServer;
use aiway_protocol::rmcp::model::{
    ClientCapabilities, ClientInfo, ClientRequest, Implementation, InitializeRequestParams,
    JsonRpcMessage, JsonRpcRequest, RequestId, ServerResult,
};
use aiway_protocol::rmcp::service::RunningService;
use aiway_protocol::rmcp::transport::StreamableHttpClientTransport;
use aiway_protocol::rmcp::transport::streamable_http_client::StreamableHttpClientTransportConfig;
use aiway_protocol::rmcp::{RoleClient, ServiceExt};
use anyhow::bail;
use dashmap::DashMap;
use http::{HeaderMap, HeaderName, HeaderValue, Method};
use serde_json::json;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::atomic::{AtomicI64, AtomicU64};
use std::sync::{Arc, LazyLock};

type ProxyClient = RunningService<RoleClient, InitializeRequestParams>;
#[derive(Debug)]
pub struct McpProxyPool {
    clients: DashMap<String, Arc<ProxyClient>>,
}
impl McpProxyPool {
    pub fn new() -> Self {
        McpProxyPool {
            clients: DashMap::new(),
        }
    }
    pub async fn get_proxy_client(&self, server_name: &str) -> anyhow::Result<Arc<ProxyClient>> {
        if let Some(client) = self.clients.get(server_name) {
            if !client.is_closed() {
                log::info!("Get proxy client from pool: {}", server_name);
                return Ok(client.clone());
            }
        }

        log::info!("Create new proxy client: {}", server_name);

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
        let headers = HashMap::from_iter(proxy_config.headers.iter().map(|(k, v)| {
            let name = HeaderName::from_str(k).expect("Invalid header name");
            let value = HeaderValue::from_str(v).expect("Invalid header value");
            (name, value)
        }));

        let config =
            StreamableHttpClientTransportConfig::with_uri(url.as_str()).custom_headers(headers);
        let transport = StreamableHttpClientTransport::from_config(config);

        let client_info = ClientInfo::new(
            ClientCapabilities::default(),
            Implementation::new("aiway mcp client", "0.1.0"),
        );
        let client = client_info.serve(transport).await?;
        let client = Arc::new(client);
        self.clients.insert(server_name.to_string(), client.clone());

        Ok(client.clone())
    }

    pub async fn request(
        &self,
        server_name: &str,
        request: JsonRpcRequest,
    ) -> anyhow::Result<McpRes<ServerResult>> {
        let client = self.get_proxy_client(server_name).await?;
        let req: ClientRequest = serde_json::from_str(&serde_json::to_string(&request)?)?;
        let res = client.send_request(req).await?;
        Ok(McpRes::success(res, request.id))
    }

    pub fn remove_proxy_client(&self, server_name: &str) {
        self.clients.remove(server_name);
    }
}

pub static MCP_PROXY_POOL: LazyLock<McpProxyPool> = LazyLock::new(McpProxyPool::new);
