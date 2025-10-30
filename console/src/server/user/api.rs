use crate::server::auth::UserPrincipal;
use crate::server::user::request::{LoginReq, UpdatePasswordReq, UserAddReq};
use crate::server::user::response::{LoginRes, UserCenterRes, UserListRes};
use crate::server::user::{UserListReq, service};
use protocol::common::req::IdsReq;
use protocol::common::res::{PageRes, Res};
use rocket::serde::json::Json;
use rocket::{get, post, routes};

pub fn routes() -> Vec<rocket::Route> {
    routes![login, logout, update_password, center, list, add, delete]
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
#[post("/update/password", data = "<req>")]
async fn update_password(req: Json<UpdatePasswordReq>, user: UserPrincipal) -> Res<()> {
    match service::update_password(req.into_inner(), user).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

// TODO 修改基本信息

/// 用户中心
#[get("/center")]
async fn center(user: UserPrincipal) -> Res<UserCenterRes> {
    match service::center(user).await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}

#[post("/manage/list", data = "<req>")]
async fn list(req: Json<UserListReq>, user: UserPrincipal) -> Res<PageRes<UserListRes>> {
    match service::list(req.0, user).await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}

#[post("/manage/add", data = "<req>")]
async fn add(req: Json<UserAddReq>, user: UserPrincipal) -> Res<()> {
    match service::add(req.into_inner(), user).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

#[post("/manage/delete", data = "<ids>")]
async fn delete(ids: Json<IdsReq>, user: UserPrincipal) -> Res<()> {
    match service::delete(ids.into_inner(), user).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}
