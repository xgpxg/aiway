use crate::server::auth::UserPrincipal;
use crate::server::system::notify::request::NotifyConfigUpdateReq;
use crate::server::system::notify::service;
use protocol::common::res::Res;
use protocol::gateway::alert::AlertConfig;
use rocket::serde::json::Json;
use rocket::{get, post};

/// 更新通知和提醒配置
#[post("/notify/config/update", data = "<req>")]
pub async fn update(req: Json<NotifyConfigUpdateReq>, _user: UserPrincipal) -> Res<()> {
    match service::update(req.0).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 获取通知和提醒配置
#[get("/notify/config")]
pub async fn get(_user: UserPrincipal) -> Res<AlertConfig> {
    match service::get().await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}
