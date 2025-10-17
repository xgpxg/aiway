use crate::server::auth::UserPrincipal;
use crate::server::user::request::{LoginReq, UpdatePasswordReq};
use crate::server::user::response::{LoginRes, UserCenterRes};
use crate::server::user::service;
use protocol::common::res::Res;
use rocket::serde::json::Json;
use rocket::{get, post, routes};

pub fn routes() -> Vec<rocket::Route> {
    routes![login, logout, update_password, center,]
}

/// 用户登录
#[post("/login", data = "<req>")]
async fn login(req: Json<LoginReq>) -> Res<LoginRes> {
    match service::login(req.into_inner()).await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 登出
#[post("/logout")]
async fn logout(user: UserPrincipal) -> Res<()> {
    match service::logout(user).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 修改密码
#[post("/updatePassword", data = "<req>")]
async fn update_password(req: Json<UpdatePasswordReq>, user: UserPrincipal) -> Res<()> {
    match service::update_password(req.into_inner(), user).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 用户中心
#[get("/center")]
async fn center(user: UserPrincipal) -> Res<UserCenterRes> {
    match service::center(user).await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}
