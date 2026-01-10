use crate::server::auth::UserPrincipal;
use crate::server::key::request::ApiKeyAddOrUpdateReq;
use crate::server::key::response::ApiKeyListRes;
use crate::server::key::{ApiKeyListReq, service};
use aiway_protocol::common::req::IdsReq;
use aiway_protocol::common::res::{PageRes, Res};
use rocket::serde::json::Json;
use rocket::{post, routes};

pub fn routes() -> Vec<rocket::Route> {
    routes![add, delete, list]
}

/// 新增密钥
#[post("/add", data = "<req>")]
pub async fn add(req: Json<ApiKeyAddOrUpdateReq>, user: UserPrincipal) -> Res<()> {
    match service::add(req.0, user).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 删除密钥
#[post("/delete", data = "<req>")]
pub async fn delete(req: Json<IdsReq>, _user: UserPrincipal) -> Res<()> {
    match service::delete(req.0).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

#[post("/list", data = "<req>")]
pub async fn list(req: Json<ApiKeyListReq>, _user: UserPrincipal) -> Res<PageRes<ApiKeyListRes>> {
    match service::list(req.0).await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}
