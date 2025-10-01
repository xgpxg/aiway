//! # ❌ 废弃
//!
//! # 网关错误处理端点
//!
//! 在Fairing阶段发生错误，需要返回错误响应时，由于Fairing不能直接返回响应，
//! 所以转发到这些错误处理的端点处理。
//!
//! ## 502
//! 网关内部错误
//!
//! ## 503
//! 服务错误
//!
//! ## 401
//! API Key验证失败
//!
use crate::openapi::error::GatewayError;
use crate::openapi::response::GatewayResponse;
use rocket::{get, routes};

pub fn routes() -> Vec<rocket::Route> {
    routes![error_502, error_503, error_401,error_403]
}

#[get("/502")]
async fn error_502() -> GatewayResponse {
    GatewayResponse::Error(GatewayError::BadGateway)
}

#[get("/503")]
async fn error_503() -> GatewayResponse {
    GatewayResponse::Error(GatewayError::ServiceUnavailable)
}

#[get("/401")]
async fn error_401() -> GatewayResponse {
    GatewayResponse::Error(GatewayError::Unauthorized)
}

#[get("/403")]
async fn error_403() -> GatewayResponse {
    GatewayResponse::Error(GatewayError::Forbidden)
}
