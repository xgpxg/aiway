use crate::server::auth::UserPrincipal;
use crate::server::cert::request::{CertIssueReq, CertListReq, SetAutoRenewReq};
use crate::server::cert::response::{CertDetailRes, CertKeyRes, CertListRes};
use crate::server::cert::service;
use busi::req::{IdReq, IdsReq};
use busi::res::{PageRes, Res};
use rocket::serde::json::Json;
use rocket::{post, routes};

pub fn routes() -> Vec<rocket::Route> {
    routes![issue, list, detail, delete, renew, set_auto_renew, key]
}

/// 签发证书
#[post("/issue", data = "<req>")]
pub async fn issue(req: Json<CertIssueReq>, user: UserPrincipal) -> Res<i64> {
    match service::issue(req.0, user).await {
        Ok(id) => Res::success(id),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 证书列表
#[post("/list", data = "<req>")]
pub async fn list(req: Json<CertListReq>, _user: UserPrincipal) -> Res<PageRes<CertListRes>> {
    match service::list(req.0).await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 证书详情
#[post("/detail", data = "<req>")]
pub async fn detail(req: Json<IdReq>, _user: UserPrincipal) -> Res<CertDetailRes> {
    match service::detail(req.0.id).await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 删除证书
#[post("/delete", data = "<req>")]
pub async fn delete(req: Json<IdsReq>, _user: UserPrincipal) -> Res<()> {
    match service::delete(req.0).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 续期证书
#[post("/renew", data = "<req>")]
pub async fn renew(req: Json<IdReq>, user: UserPrincipal) -> Res<i64> {
    match service::renew(req.0, user).await {
        Ok(id) => Res::success(id),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 设置自动续期
#[post("/set_auto_renew", data = "<req>")]
pub async fn set_auto_renew(req: Json<SetAutoRenewReq>, _user: UserPrincipal) -> Res<()> {
    match service::set_auto_renew(req.0).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 获取证书和私钥
#[post("/key", data = "<req>")]
pub async fn key(req: Json<IdReq>, _user: UserPrincipal) -> Res<CertKeyRes> {
    match service::get_cert_key(req.0.id).await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}
