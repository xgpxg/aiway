use crate::server::auth::UserPrincipal;
use crate::server::system::acme::request::AcmeConfigUpdateReq;
use crate::server::system::acme::service;
use crate::server::system::acme::AcmeConfig;
use busi::res::Res;
use rocket::serde::json::Json;
use rocket::{get, post};

/// 更新 ACME 配置
#[post("/acme/config/update", data = "<req>")]
pub async fn update(req: Json<AcmeConfigUpdateReq>, _user: UserPrincipal) -> Res<()> {
    match service::update(req.0).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 获取 ACME 配置（凭证脱敏）
#[get("/acme/config")]
pub async fn get(_user: UserPrincipal) -> Res<AcmeConfig> {
    match service::get().await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}
