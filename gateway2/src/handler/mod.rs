//! # Fairing 模块 - Pingora 中间件实现
//!
//! 对应原 Rocket Fairing 的功能，使用 Pingora 的拦截器模式实现

pub mod auth;
pub mod cleanup;
pub mod filter;
pub mod global_filter;
pub mod lb;
pub mod logger;
pub mod pre;
pub mod request;
pub mod response;
pub mod routing;
pub mod security;

pub use auth::auth_handle;
pub use cleanup::cleanup_handle;
pub use filter::{post_filter, pre_filter};
pub use global_filter::{global_post_filter, global_pre_filter};
use http::header::ToStrError;
pub use lb::lb_handle;
pub use logger::log_handle;
use pingora::{BError, Error};
pub use pre::pre_handle;
pub use request::request_handle;
pub use response::response_handle;
pub use routing::routing_handle;
pub use security::security_check;

pub(crate) type HttpResult<T> = std::result::Result<T, HttpError>;
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
