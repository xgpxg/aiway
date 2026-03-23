use crate::components::Servicer;
use crate::mcp_proxy::components::McpFactory;
use crate::mcp_proxy::proxy::proxy_pool::MCP_PROXY_POOL;
use crate::mcp_proxy::proxy::response::McpRes;
use aiway_protocol::mcp::mcp::{McpServerType, Route, RouteType};
use aiway_protocol::rmcp::model::{
    CallToolRequestParams, CallToolResult, Content, JsonObject, JsonRpcRequest, ListToolsResult,
    ServerCapabilities, ServerResult,
};
use aiway_protocol::rmcp::model::{InitializeResult, Tool};
use http::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::LazyLock;
use std::time::Duration;

pub struct HttpClient {
    pub client: Client,
}

static HTTP_CLIENT: LazyLock<HttpClient> = LazyLock::new(HttpClient::new);
impl HttpClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        HttpClient { client }
    }
}

pub struct McpServerHandler;
impl McpServerHandler {
    /// 初始化
    pub async fn initialize(
        name: &str,
        request: JsonRpcRequest,
    ) -> anyhow::Result<McpRes<ServerResult>> {
        let id = request.id;
        let exists = McpFactory::server_exists(name);
        if !exists {
            return Ok(McpRes::error(&format!("Mcp server {} not found", name), id));
        }
        // 目前只支持tool
        let data = InitializeResult::new(ServerCapabilities::builder().enable_tools().build());
        Ok(McpRes::success(ServerResult::InitializeResult(data), id))
    }
    pub async fn list_tools(
        name: &str,
        request: JsonRpcRequest,
    ) -> anyhow::Result<McpRes<ServerResult>> {
        let id = request.id;
        let server = McpFactory::get_server(name).await;
        match server {
            None => Ok(McpRes::error(&format!("Mcp server {} not found", name), id)),
            Some(server) => {
                let tools = server
                    .tools
                    .values()
                    .map(|mcp_tool| mcp_tool.clone().into())
                    .collect::<Vec<Tool>>();
                let data = ListToolsResult {
                    tools,
                    ..Default::default()
                };
                Ok(McpRes::success(ServerResult::ListToolsResult(data), id))
            }
        }
    }

    pub async fn call_tool(
        name: &str,
        request: JsonRpcRequest,
    ) -> anyhow::Result<McpRes<ServerResult>> {
        let id = request.id.clone();
        let server = McpFactory::get_server(name).await;
        match server {
            // Factory中不存在，返回错误
            None => Ok(McpRes::error(&format!("Mcp server {} not found", name), id)),
            Some(server) => {
                let params = serde_json::from_value::<CallToolRequestParams>(serde_json::json!(
                    request.request.params
                ))?;
                let tool_name = &params.name;
                let arguments = &params.arguments;

                let mcp_tool = McpFactory::get_tool(name, tool_name).await;
                if mcp_tool.is_none() {
                    return Ok(McpRes::error(
                        &format!("Mcp tool {} not found", tool_name),
                        id,
                    ));
                }
                let mcp_tool = mcp_tool.unwrap();
                let server_type = server.server_type;

                match server_type {
                    // 执行接口调用
                    McpServerType::Http => {
                        let route = mcp_tool.route;
                        let method = match route.method.as_ref() {
                            Some(method) => method,
                            None => "GET",
                        };
                        let url = match route.route_type {
                            // 指定服务名，从Load-balance中获取实例
                            RouteType::Service => {
                                let service_name = route
                                    .service_name
                                    .as_ref()
                                    .expect("Tool route type is Service, but service name is none");
                                let path = route
                                    .service_path
                                    .as_ref()
                                    .map(|path| path.to_string())
                                    .unwrap_or_default();
                                let url = Servicer::get_instance(&service_name)
                                    .expect("No service instance found");
                                format!("{}{}", url, path)
                            }
                            // 直接指定的URL
                            RouteType::Url => route.url.clone().unwrap(),
                            _ => {
                                unimplemented!()
                            }
                        };
                        let (query, body, header) = extract_parameters(&route, &arguments);
                        let res = HTTP_CLIENT
                            .client
                            .request(http::Method::from_str(method)?, url)
                            .query(&query)
                            .json(&body)
                            .headers(HeaderMap::from_iter(header.iter().map(|(k, v)| {
                                let name = HeaderName::from_str(k).expect("Invalid header name");
                                let value = HeaderValue::from_str(v.as_str().unwrap())
                                    .expect("Invalid header value");
                                (name, value)
                            })))
                            .send()
                            .await;
                        log::info!("call api res: {:?}", res);
                        let res = match res {
                            Ok(res) => res.text().await?,
                            Err(e) => {
                                return Ok(McpRes::error(&format!("{}", e), id));
                            }
                        };
                        let content = Content::text(res);
                        let data = CallToolResult::success(vec![content]);
                        Ok(McpRes::success(ServerResult::CallToolResult(data), id))
                    }
                    // 执行代理调用
                    McpServerType::Proxy => {
                        let res = MCP_PROXY_POOL
                            .request(&server.name, request)
                            .await
                            .map_err(|e| {
                                log::error!("Mcp proxy error: {}", e);
                                e
                            })?;
                        Ok(res)
                    }
                    _ => {
                        unimplemented!()
                    }
                }
            }
        }
    }
}

/// 从MCP请求参数中提取参数
fn extract_parameters(
    api: &Route,
    arguments: &Option<JsonObject>,
) -> (
    HashMap<String, Value>,
    HashMap<String, Value>,
    HashMap<String, Value>,
) {
    let mut query_params = HashMap::new();
    let mut body_params = HashMap::new();
    let mut header_params = HashMap::new();

    if let Some(params) = api.request_param.clone() {
        let params = params.as_array().unwrap_or(&vec![]).clone();
        for param in params {
            let name = param["name"].as_str().unwrap_or_default();
            let position = param["position"].as_str().unwrap_or_default();

            if let Some(value) = arguments.as_ref().and_then(|args| args.get(name)) {
                match position {
                    "url" => {
                        query_params.insert(name.to_string(), value.clone());
                    }
                    "body" => {
                        body_params.insert(name.to_string(), value.clone());
                    }
                    "header" => {
                        header_params.insert(name.to_string(), value.clone());
                    }
                    _ => {}
                }
            }
        }
    }

    (query_params, body_params, header_params)
}
