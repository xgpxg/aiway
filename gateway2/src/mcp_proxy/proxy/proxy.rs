use bytes::Bytes;
use crate::mcp_proxy::proxy::handler::McpServerHandler;
use crate::mcp_proxy::proxy::response::{EmptyResponse, McpRes};
use aiway_protocol::rmcp::model::{
    JsonRpcMessage, JsonRpcNotification, JsonRpcRequest, RequestId, ServerResult,
};

/// MCP 服务实现
///
/// 支持2种MCP模式：
/// - HTTP模式：将HTTP接口伪装为MCP工具
/// - 代理模式：直接将MCP客户端的请求转发到已有的MCP服务，仅支持 Streamable HTTP，不支持纯SSE。
///
/// 返回 `reqwest::Response` ，走网关的标准处理流程
pub async fn mcp(mcp_server: &str, body: Bytes) -> reqwest::Response {
    // url解码
    let mcp_server = match urlencoding::decode(mcp_server) {
        Ok(s) => s.to_string(),
        Err(e) => {
            log::error!("Decode url error: {}", e);
            return McpRes::<()>::error(&e.to_string(), RequestId::Number(0)).into();
        }
    };
    log::debug!("Request mcp server: {}", mcp_server);
    // 解析请求体为JSON-RPC消息
    let message = match serde_json::from_slice::<JsonRpcMessage>(body.as_ref()) {
        Ok(message) => message,
        Err(e) => {
            log::error!("Parse mcp request error: {}", e);
            return McpRes::<()>::error(&e.to_string(), RequestId::Number(0)).into();
        }
    };

    log::debug!("Receive mcp client message: {:?}", message);

    match message {
        // 处理客户端请求
        JsonRpcMessage::Request(request) => {
            let id = request.id.clone();
            match handle_mcp_request(&mcp_server, request).await {
                Ok(res) => res.into(),
                Err(e) => McpRes::<()>::error(&e.to_string(), id).into(),
            }
        }
        JsonRpcMessage::Response(_) => {
            unimplemented!()
        }
        // 处理客户端通知
        // notification不需要返回响应，仅打印错误日志即可
        JsonRpcMessage::Notification(notification) => {
            if let Err(e) = handle_mcp_notification(&mcp_server, notification).await {
                log::error!("Handle notification error: {}", e);
            }
            EmptyResponse.into()
        }
        JsonRpcMessage::Error(_) => {
            unimplemented!()
        }
    }
}

/// 处理客户端请求
pub async fn handle_mcp_request(
    mcp_server: &str,
    request: JsonRpcRequest,
) -> anyhow::Result<McpRes<ServerResult>> {
    let method = request.request.method.as_str();
    match method {
        // 初始化
        "initialize" => McpServerHandler::initialize(mcp_server, request).await,
        // 获取工具列表
        "tools/list" => McpServerHandler::list_tools(mcp_server, request).await,
        "tools/call" => McpServerHandler::call_tool(mcp_server, request).await,
        _ => unimplemented!(),
    }
}

pub async fn handle_mcp_notification(
    _mcp_server: &str,
    notification: JsonRpcNotification,
) -> anyhow::Result<()> {
    let method = notification.notification.method;
    match method.as_str() {
        "notifications/initialized" => {
            log::info!("Receive notification: {}", method);
        }
        "notifications/cancelled" => {
            log::info!("Receive notification: {}", method);
        }
        _ => unimplemented!(),
    }
    Ok(())
}
