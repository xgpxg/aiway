use crate::server::auth::UserPrincipal;
use crate::server::metrics::request::RegionRequestCountReq;
use crate::server::metrics::response::{GatewayState, RegionRequestCountRes};
use crate::server::metrics::service;
use protocol::common::res::Res;
use rocket::serde::json::Json;
use rocket::{get, post, routes};

pub fn routes() -> Vec<rocket::Route> {
    routes![gateway_state, request_region_count]
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
