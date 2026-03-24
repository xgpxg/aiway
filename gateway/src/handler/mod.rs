pub mod auth;
pub mod cleanup;
pub mod lb;
pub mod logger;
pub mod plugin;
pub mod pre;
pub mod response;
pub mod routing;
pub mod security;

use aiway_protocol::context::parts::SerdeParts;
pub use auth::auth_handle;
pub use cleanup::cleanup_handle;
use http::header::ToStrError;
pub use lb::lb_handle;
pub use logger::log_handle;
use pingora::BError;
use pingora::protocols::http::ServerSession;
pub use pre::pre_handle;
pub use response::response_handle;
pub use routing::routing_handle;
pub use security::firewall_check;
pub(crate) type HandlerResult<T> = Result<T, HandlerError>;
#[derive(Debug)]
pub struct HandlerError(pub(crate) u16, pub(crate) String);
impl HandlerError {
    pub fn new(code: u16, message: &str) -> Self {
        HandlerError(code, message.to_string())
    }
}
impl std::fmt::Display for HandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.0, self.1)
    }
}

impl From<ToStrError> for HandlerError {
    fn from(e: ToStrError) -> Self {
        HandlerError(500, e.to_string())
    }
}
impl From<BError> for HandlerError {
    fn from(value: BError) -> Self {
        HandlerError(500, value.to_string())
    }
}

/// 从状态码生成错误响应
pub fn error_resp_from_status_code(stats_code: u16) -> SerdeParts {
    let resp = ServerSession::generate_error(stats_code);
    SerdeParts {
        method: None,
        status_code: Some(resp.status.clone()),
        uri: None,
        headers: Some(resp.headers.clone()),
        authority: None,
    }
}
/// 响应错误并结束处理。
/// 仅适用于 `request_filter` 阶段
pub async fn respond_error_end(
    session: &mut ServerSession,
    ctx: &mut aiway_protocol::context::HttpContext,
    error: HandlerError,
) -> pingora::Result<bool> {
    respond_error(session, ctx, error).await?;
    Ok(true)
}

/// 响应错误
pub async fn respond_error(
    session: &mut ServerSession,
    ctx: &mut aiway_protocol::context::HttpContext,
    error: HandlerError,
) -> pingora::Result<()> {
    ctx.insert_state(
        aiway_protocol::context::HttpContext::RESPONSE_SERDE_PARTS,
        error_resp_from_status_code(error.0),
    );
    session
        .respond_error_with_body(error.0, error.1.into())
        .await?;
    Ok(())
}