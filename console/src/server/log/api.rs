use crate::args::Args;
use crate::server::auth::UserPrincipal;
use crate::server::log::request::LogListReq;
use crate::server::log::service;
use protocol::common::res::{PageRes, Res};
use protocol::logg::LogEntry;
use rocket::serde::json::Json;
use rocket::{State, post, routes};

pub fn routes() -> Vec<rocket::Route> {
    routes![list]
}

/// 查询日志
#[post("/list", data = "<req>")]
pub async fn list(
    req: Json<LogListReq>,
    _user: UserPrincipal,
    args: &State<Args>,
) -> Res<PageRes<LogEntry>> {
    match service::list(req.0, args).await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}
