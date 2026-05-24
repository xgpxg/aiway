use bytes::Bytes;
mod handler;
#[allow(clippy::module_inception)]
mod proxy;
mod proxy_pool;

mod response;

pub use proxy_pool::MCP_PROXY_POOL;

#[allow(unused)]
pub async fn mcp_get(_mcp_server: String) {

}

pub async fn mcp_post(path: &str, body: Bytes) -> reqwest::Result<reqwest::Response> {
    // 提取server: /v1/mcp/<server>
    let mcp_server = path.split("/").nth(3).unwrap();
    let response = proxy::mcp(mcp_server, body).await;
    Ok(response)
}
