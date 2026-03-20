//! # 网关错误类型
//!
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GatewayError {
    #[error("Bad Gateway")]
    BadGateway,
    
    #[error("Service Unavailable")]
    ServiceUnavailable,
    
    #[error("Not Found")]
    NotFound,
    
    #[error("Unauthorized")]
    Unauthorized,
    
    #[error("Forbidden")]
    Forbidden,
}
