use crate::server::auth::UserPrincipal;
use crate::server::common::pool::HTTP_CLIENT;
use crate::server::db::models::mcp_server::{McpServer, McpServerBuilder, McpServerStatus};
use crate::server::db::models::mcp_tool;
use crate::server::db::models::mcp_tool::{McpTool, McpToolBuilder, McpToolStatus};
use crate::server::db::{Pool, tools};
use crate::server::mcp::request::{
    McpServerAddReq, McpServerStatusUpdateReq, McpServerUpdateReq, McpToolAddReq, McpToolUpdateReq,
    SyncProxyServerToolsReq, UpdateMcpToolStatusReq,
};
use crate::server::mcp::response::{McpServerListRes, McpToolListRes};
use crate::server::mcp::{McpServerListReq, McpToolListReq};
use aiway_protocol::mcp::mcp::RouteType;
use aiway_protocol::rmcp::model::{
    InitializeRequest, InitializeRequestParams, JsonRpcMessage, JsonRpcResponse, ListToolsRequest,
    Notification, RequestId,
};
use aiway_protocol::rmcp::object;
use anyhow::bail;
use busi::req::{IdReq, IdsReq, Pagination};
use busi::res::{IntoPageRes, PageRes};
use common::id;
use logging::log;
use rbs::value;
use reqwest::Method;
use serde_json::{Value, json};

pub async fn server_add(req: McpServerAddReq, user: UserPrincipal) -> anyhow::Result<()> {
    let server = McpServerBuilder::default()
        .id(id::next().into())
        .name(req.name.into())
        .server_type(req.server_type.into())
        .description(req.description)
        .status(McpServerStatus::Disable.into())
        .create_user_id(Some(user.id))
        .create_time(Some(tools::now()))
        .build()?;

    if check_server_exists(&server, None).await? {
        bail!(
            "MCP Server with name {} already exists",
            server.name.unwrap()
        )
    }
    McpServer::insert(Pool::get()?, &server).await?;
    Ok(())
}

async fn check_server_exists(server: &McpServer, exclude_id: Option<i64>) -> anyhow::Result<bool> {
    let mut list = McpServer::select_by_map(
        Pool::get()?,
        value! {
            "name": &server.name,
        },
    )
    .await?;

    list.retain(|item| item.id != exclude_id);

    Ok(!list.is_empty())
}

pub async fn server_list(_req: McpServerListReq) -> anyhow::Result<Vec<McpServerListRes>> {
    let mut list = McpServer::select_all(Pool::get()?).await?;
    list.sort_by(|a, b| b.id.cmp(&a.id));

    let list = list
        .into_iter()
        .map(|item| McpServerListRes { inner: item })
        .collect::<Vec<_>>();

    Ok(list)
}

pub async fn server_update(req: McpServerUpdateReq, user: UserPrincipal) -> anyhow::Result<()> {
    let old = McpServer::select_by_map(Pool::get()?, value! { "id": req.id}).await?;
    if old.is_empty() {
        bail!("MCP Server not found");
    }

    let old = old.first().unwrap();

    if check_server_exists(&old, Some(req.id)).await? {
        bail!(
            "MCP Server with name {} already exists",
            old.name.as_ref().unwrap()
        )
    }

    let update = McpServerBuilder::default()
        .id(req.id.into())
        .name(req.name)
        .description(req.description)
        .server_type(req.server_type)
        .proxy_config(req.proxy_config)
        .update_user_id(Some(user.id))
        .update_time(Some(tools::now()))
        .build()?;

    McpServer::update_by_map(Pool::get()?, &update, value! { "id":req.id}).await?;
    Ok(())
}

pub async fn server_update_status(
    req: McpServerStatusUpdateReq,
    user: UserPrincipal,
) -> anyhow::Result<()> {
    let old = McpServer::select_by_map(Pool::get()?, value! { "id": req.id}).await?;
    if old.is_empty() {
        bail!("MCP Server not found")
    }
    McpServer::update_by_map(
        Pool::get()?,
        &McpServerBuilder::default()
            .id(Some(req.id))
            .status(Some(req.status))
            .update_user_id(Some(user.id))
            .update_time(Some(tools::now()))
            .build()?,
        value! { "id": req.id},
    )
    .await?;
    Ok(())
}

pub async fn server_delete(req: IdsReq) -> anyhow::Result<()> {
    // 删除工具
    McpTool::delete_by_map(Pool::get()?, value! { "mcp_server_id": &req.ids}).await?;
    // 删除服务
    McpServer::delete_by_map(Pool::get()?, value! { "id": &req.ids}).await?;
    Ok(())
}

// ==================== MCP Tool Service ====================

pub async fn tool_add(req: McpToolAddReq, user: UserPrincipal) -> anyhow::Result<()> {
    let tool = McpToolBuilder::default()
        .id(id::next().into())
        .mcp_server_id(req.mcp_server_id.into())
        .name(req.name.into())
        .description(req.description)
        .input_schema(req.input_schema)
        .output_schema(req.output_schema)
        .route_type(req.route_type.into())
        .service_name(req.service_name)
        .service_path(req.service_path)
        .url(req.url)
        .method(req.method)
        .request_param(req.request_param)
        .status(Some(McpToolStatus::Disable))
        .create_user_id(Some(user.id))
        .create_time(Some(tools::now()))
        .build()?;

    if check_tool_exists(&tool, None).await? {
        bail!("MCP Tool with name {} already exists", tool.name.unwrap())
    }
    McpTool::insert(Pool::get()?, &tool).await?;
    Ok(())
}

async fn check_tool_exists(tool: &McpTool, exclude_id: Option<i64>) -> anyhow::Result<bool> {
    let mut list = McpTool::select_by_map(
        Pool::get()?,
        value! {
            "name": &tool.name,
        },
    )
    .await?;

    list.retain(|item| item.id != exclude_id);

    Ok(!list.is_empty())
}

pub async fn tool_list(req: McpToolListReq) -> anyhow::Result<PageRes<McpToolListRes>> {
    let page = mcp_tool::list_page(Pool::get()?, &req.to_rb_page(), &req).await?;
    let list = page.convert_to_page_res(|list| {
        list.into_iter()
            .map(|item| McpToolListRes { inner: item })
            .collect::<Vec<_>>()
    });
    Ok(list)
}

pub async fn tool_update(req: McpToolUpdateReq, user: UserPrincipal) -> anyhow::Result<()> {
    let old = McpTool::select_by_map(Pool::get()?, value! { "id": req.id}).await?;
    if old.is_empty() {
        bail!("MCP Tool not found");
    }

    let old = old.first().unwrap();
    if check_tool_exists(&old, Some(req.id)).await? {
        bail!(
            "MCP Tool with name {} already exists",
            old.name.as_ref().unwrap()
        )
    }
    let update = McpToolBuilder::default()
        .id(req.id.into())
        .description(req.description)
        .input_schema(req.input_schema)
        .output_schema(req.output_schema)
        .route_type(req.route_type)
        .service_name(req.service_name)
        .service_path(req.service_path)
        .url(req.url)
        .method(req.method)
        .request_param(req.request_param)
        .update_user_id(Some(user.id))
        .update_time(Some(tools::now()))
        .build()?;

    McpTool::update_by_map(Pool::get()?, &update, value! { "id":req.id}).await?;
    Ok(())
}

pub async fn tool_delete(req: IdsReq) -> anyhow::Result<()> {
    McpTool::delete_by_map(Pool::get()?, value! { "id": req.ids}).await?;
    Ok(())
}

pub async fn tool_update_status(
    req: UpdateMcpToolStatusReq,
    user: UserPrincipal,
) -> anyhow::Result<()> {
    let old = McpTool::select_by_map(Pool::get()?, value! { "id": req.id}).await?;
    if old.is_empty() {
        bail!("MCP Tool not found")
    }
    McpTool::update_by_map(
        Pool::get()?,
        &McpToolBuilder::default()
            .id(Some(req.id))
            .status(Some(req.status))
            .update_user_id(Some(user.id))
            .update_time(Some(tools::now()))
            .build()?,
        value! { "id": req.id},
    )
    .await?;
    Ok(())
}

pub async fn sync_proxy_server_tools(req: IdReq, user: UserPrincipal) -> anyhow::Result<()> {
    let id = req.id;
    let mcp_server = McpServer::select_by_map(Pool::get()?, value! { "id": id}).await?;
    if mcp_server.is_empty() {
        bail!("MCP Server not found")
    }
    let mcp_server = mcp_server.first().unwrap();

    let proxy_config = mcp_server.proxy_config.as_ref().unwrap();
    let url = &proxy_config.url;
    let res = HTTP_CLIENT
        .request(Method::POST, url)
        .header("accept", "application/json, text/event-stream")
        .json(&json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{},"clientInfo":{"name":"rmcp","version":"1.2.0"},"protocolVersion":"2025-06-18"}}))
        .send()
        .await?;

    let mcp_session_id = res
        .headers()
        .get("mcp-session-id")
        .map(|h| h.to_str().unwrap().to_string())
        .unwrap_or_default();

    let _ = HTTP_CLIENT
        .request(Method::POST, url)
        .header("accept", "application/json, text/event-stream")
        .header("mcp-session-id", mcp_session_id.clone())
        .json(&json!({"jsonrpc":"2.0","method":"notifications/initialized","params":{}}))
        .send()
        .await?;

    let res = HTTP_CLIENT
        .request(Method::POST, url)
        .header("accept", "application/json, text/event-stream")
        .header("mcp-session-id", mcp_session_id)
        .json(&json!({"method":"tools/list","jsonrpc":"2.0","id":2}))
        .send()
        .await?;
    let result = &res.json::<Value>().await?["result"];

    let tools = result["tools"].as_array().unwrap();

    let mut mcp_tools = vec![];
    for tool in tools.into_iter() {
        log::info!("sync tool: {:?}", tool);

        let name = tool["name"].as_str().unwrap();
        let description = tool["description"].as_str().unwrap();
        let input_schema = tool["inputSchema"].clone();

        mcp_tools.push(
            McpToolBuilder::default()
                .id(id::next().into())
                .mcp_server_id(mcp_server.id)
                .name(name.to_string().into())
                .description(description.to_string().into())
                .input_schema(input_schema.into())
                .route_type(RouteType::Url.into())
                .status(McpToolStatus::Ok.into())
                .create_time(tools::now().into())
                .create_user_id(user.id.into())
                .build()?,
        );
    }

    // 先删除
    McpTool::delete_by_map(Pool::get()?, value! { "mcp_server_id": id}).await?;
    McpTool::insert_batch(Pool::get()?, &mcp_tools, 10).await?;

    Ok(())
}
