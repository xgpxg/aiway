use aiway_protocol::context::HttpContext;
use context::HttpContextWrapper;

mod handler;
mod proxy;
mod proxy_pool;

mod response;

pub use proxy_pool::MCP_PROXY_POOL;

pub async fn mcp_get(mcp_server: String, context: HttpContextWrapper) {
    println!("MCP GET");
    println!("MCP请求: {:?}", context);
}

pub async fn mcp_post(context: &HttpContext) -> reqwest::Result<reqwest::Response> {
    // 提取server
    let path = context.request.get_path();
    let mcp_server = path.split("/").nth(2).unwrap();
    let response = proxy::mcp(mcp_server, context).await;
    Ok(reqwest::Response::from(response))
}
