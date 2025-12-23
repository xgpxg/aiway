use crate::server::model_proxy::model;
use protocol::common::res::Res;
use rocket::{get, routes};

pub fn routes() -> Vec<rocket::Route> {
    routes![all_models]
}

/// 获取所有模型，仅由`model-proxy`服务调用
#[get("/model/models")]
async fn all_models() -> Res<Vec<protocol::model::Model>> {
    match model::models().await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}
