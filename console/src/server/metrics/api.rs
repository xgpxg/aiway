use crate::server::auth::UserPrincipal;
use crate::server::metrics::request::{RegionRequestCountReq, RequestStatusCountReq};
use crate::server::metrics::response::{GatewayState, RegionRequestCountRes, RequestStatusCountRes};
use crate::server::metrics::service;
use busi::res::Res;
use rocket::serde::json::Json;
use rocket::{get, post, routes};

pub fn routes() -> Vec<rocket::Route> {
    routes![gateway_state, request_region_count, request_status_count]
}

/// 网关状态
#[get("/gateway/state")]
async fn gateway_state(_user: UserPrincipal) -> Res<GatewayState> {
    match service::gateway_state().await {
        Ok(state) => Res::success(state),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 区域（省份）调用统计
#[post("/region/count", data = "<req>")]
async fn request_region_count(
    req: Json<RegionRequestCountReq>,
    _user: UserPrincipal,
) -> Res<Vec<RegionRequestCountRes>> {
    match service::request_region_count(req.0).await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 状态码统计
#[post("/status/count", data = "<req>")]
async fn request_status_count(
    req: Json<RequestStatusCountReq>,
    _user: UserPrincipal,
) -> Res<Vec<RequestStatusCountRes>> {
    match service::request_status_count(req.0).await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}
