use crate::server;
use crate::server::gateway::{plugin, reporter, route, service};
use protocol::common::res::Res;
use rocket::serde::json::Json;
use rocket::{get, post, routes};

pub fn routes() -> Vec<rocket::Route> {
    routes![all_routes, all_services, all_plugins, configuration, report]
}

/// 查询路由表
#[get("/gateway/routes")]
async fn all_routes() -> Res<Vec<protocol::gateway::Route>> {
    match route::routes().await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 查询服务表
#[get("/gateway/services")]
async fn all_services() -> Res<Vec<protocol::gateway::Service>> {
    match service::services().await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 查询插件
///
/// 需返回稳定有序列表
#[get("/gateway/plugins")]
async fn all_plugins() -> Res<Vec<protocol::gateway::Plugin>> {
    match plugin::plugins().await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 查询配置
#[get("/gateway/configuration")]
async fn configuration() -> Res<protocol::gateway::Configuration> {
    match server::gateway::configuration::configuration().await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}

#[post("/gateway/report", data = "<req>")]
async fn report(req: Json<protocol::gateway::state::State>) -> Res<()> {
    match reporter::report(req.0).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}
