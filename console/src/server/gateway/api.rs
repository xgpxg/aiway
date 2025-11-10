//! # gateway与console交互相关接口
//!
//! 目前gateway使用定时同步的方式从console拉取配置。
//!
//! 待定：本模块中的接口目前没有做权限验证，后面需要确认请求是否来自gateway。
//!
use crate::server;
use crate::server::gateway::{alerter, plugin, reporter, route, service};
use protocol::common::res::Res;
use protocol::gateway::alert::AlertMessage;
use rocket::serde::json::Json;
use rocket::{get, post, routes};

pub fn routes() -> Vec<rocket::Route> {
    routes![
        all_routes,
        all_services,
        all_plugins,
        configuration,
        firewall,
        report,
        alert,
    ]
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
#[get("/gateway/plugins")]
async fn all_plugins() -> Res<Vec<protocol::gateway::Plugin>> {
    match plugin::plugins().await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 查询全局路由插件
#[get("/gateway/global/filter")]
async fn configuration() -> Res<protocol::gateway::GlobalFilter> {
    match server::gateway::global_filter::config().await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 查询防火墙配置
#[get("/gateway/firewall")]
async fn firewall() -> Res<protocol::gateway::Firewall> {
    match server::gateway::firewall::configuration().await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 接收状态上报
#[post("/gateway/report", data = "<req>")]
async fn report(req: Json<protocol::gateway::state::State>) -> Res<()> {
    match reporter::report(req.0).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 接收并推送告警消息
#[post("/gateway/alert", data = "<req>")]
async fn alert(req: Json<AlertMessage>) -> Res<()> {
    match alerter::alert(req.0).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}
