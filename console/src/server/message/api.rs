use crate::server::auth::UserPrincipal;
use crate::server::message::request::MessageListReq;
use crate::server::message::response::{MessageCountRes, MessageListRes};
use crate::server::message::service;
use busi::req::IdReq;
use busi::res::{PageRes, Res};
use rocket::serde::json::Json;
use rocket::{post, routes};

pub fn routes() -> Vec<rocket::Route> {
    routes![count_unread, list, read, delete]
}

/// 获取未读消息数量
#[post("/count/unread")]
pub async fn count_unread(_user: UserPrincipal) -> Res<MessageCountRes> {
    match service::count_unread().await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 消息列表
#[post("/list", data = "<req>")]
pub async fn list(req: Json<MessageListReq>, _user: UserPrincipal) -> Res<PageRes<MessageListRes>> {
    match service::list(req.0).await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 标记已读
///
/// 全部已读：id传-1
#[post("/read", data = "<req>")]
pub async fn read(req: Json<IdReq>, _user: UserPrincipal) -> Res<()> {
    match service::read(req.0).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 删除消息（物理删除）
#[post("/delete", data = "<req>")]
pub async fn delete(req: Json<IdReq>, _user: UserPrincipal) -> Res<()> {
    match service::delete(req.0).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}
