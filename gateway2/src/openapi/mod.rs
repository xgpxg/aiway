//! # OpenAPI 统一入口 - Pingora 实现
//!
//! 处理所有 HTTP 方法的请求转发

pub mod client;
mod error;
mod response;

use pingora::prelude::*;
use anyhow::Context;

/// 处理 HTTP 请求
pub async fn handle_request(session: &mut Session) -> pingora::Result<()> {
    // TODO: 实现请求转发逻辑
    
    Ok(())
}
