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
pub(crate) type HttpResult<T> = Result<T, HttpError>;
#[derive(Debug)]
pub struct HttpError(pub(crate) u16, pub(crate) String);
impl HttpError {
    pub fn new(code: u16, message: &str) -> Self {
        HttpError(code, message.to_string())
    }
}
impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.0, self.1)
    }
}
impl From<(u16, String)> for HttpError {
    fn from(value: (u16, String)) -> Self {
        HttpError(value.0, value.1)
    }
}
impl From<(u16, &str)> for HttpError {
    fn from(value: (u16, &str)) -> Self {
        HttpError(value.0, value.1.to_string())
    }
}
impl From<(isize, &str)> for HttpError {
    fn from(value: (isize, &str)) -> Self {
        HttpError(value.0 as u16, value.1.to_string())
    }
}

impl From<ToStrError> for HttpError {
    fn from(e: ToStrError) -> Self {
        HttpError(500, e.to_string())
    }
}
impl From<BError> for HttpError {
    fn from(value: BError) -> Self {
        HttpError(500, value.to_string())
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

/// 响应错误并结束处理
#[macro_export]
macro_rules! respond_error_end {
    // 默认行为：记录状态并返回错误响应
    ($session:expr, $ctx:expr, $error:expr) => {
        respond_error!($session, $ctx, $error)?;
        return Ok(true);
    };
}

/// 响应错误
#[macro_export]
macro_rules! respond_error {
    ($session:expr, $ctx:expr, $error:expr) => {{
        $ctx.insert_state(
            aiway_protocol::context::HttpContext::RESPONSE_SERDE_PARTS,
            crate::handler::error_resp_from_status_code($error.0),
        );
        $session
            .respond_error_with_body($error.0, $error.1.into())
            .await
    }};
}
