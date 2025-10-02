use crate::server::metrics::service;
use protocol::common::res::Res;
use rocket::{get, routes};
use crate::server::metrics::response::GatewayState;

pub fn routes() -> Vec<rocket::Route> {
    routes![gateway_state]
}

// 网关整体运行状态
#[get("/gateway/state")]
async fn gateway_state() -> Res<GatewayState> {
    match service::gateway_state().await {
        Ok(state) => Res::success(state),
        Err(e) => Res::error(&e.to_string()),
    }
}
