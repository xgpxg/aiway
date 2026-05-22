pub mod auth;
pub mod cleanup;
pub mod lb;
pub mod logger;
pub mod plugin;
pub mod pre;
pub mod response;
pub mod routing;
pub mod security;

pub use auth::auth_handle;
pub use cleanup::cleanup_handle;
use http::header::ToStrError;
pub use lb::lb_handle;
pub use logger::log_handle;
use pingora::{BError, ErrorType};
pub use pre::pre_handle;
pub use response::response_handle;
pub use routing::routing_handle;
pub use security::firewall_check;
use std::error::Error;
pub(crate) type HandlerResult<T> = Result<T, HandlerError>;
#[derive(Debug)]
pub(crate) struct HandlerError(pub u16, pub String);

impl HandlerError {
    pub fn new(code: u16, message: &str) -> Self {
        HandlerError(code, message.to_string())
    }

    #[allow(unused)]
    pub fn code(&self) -> u16 {
        self.0
    }

    #[allow(unused)]
    pub fn message(&self) -> String {
        self.1.clone()
    }
}

impl Error for HandlerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
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

impl From<HandlerError> for BError {
    fn from(value: HandlerError) -> Self {
        pingora::Error::because(ErrorType::HTTPStatus(value.0), "", value)
    }
}

// /// 从状态码生成错误响应
// pub fn error_resp_from_status_code(stats_code: u16) -> SerdeParts {
//     let resp = ServerSession::generate_error(stats_code);
//     SerdeParts {
//         method: None,
//         status_code: Some(resp.status.clone()),
//         uri: None,
//         headers: Some(resp.headers.clone()),
//         authority: None,
//     }
// }
// /// 响应错误并结束后续处理。
// /// 仅适用于 `request_filter` 阶段
// pub async fn respond_error_end(
//     session: &mut ServerSession,
//     ctx: &mut aiway_protocol::context::HttpContext,
//     error: HandlerError,
// ) -> pingora::Result<bool> {
//     let _ = respond_error(session, ctx, error).await;
//     Ok(true)
// }

// /// 响应错误
// pub async fn respond_error(
//     session: &mut ServerSession,
//     ctx: &mut aiway_protocol::context::HttpContext,
//     error: HandlerError,
// ) -> pingora::Result<()> {
//     ctx.insert_state(
//         aiway_protocol::context::HttpContext::RESPONSE_SERDE_PARTS,
//         error_resp_from_status_code(error.0),
//     );
//     session
//         .respond_error_with_body(error.0, error.1.into())
//         .await?;
//
//     Err(pingora::Error::new_in(ErrorType::HTTPStatus(error.0)))
// }
