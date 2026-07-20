use crate::args::Args;
use crate::server::auth::UserPrincipal;
use crate::server::log::request::{DeleteLogReq, LogListReq, ModelCallLogListReq};
use crate::server::log::service;
use aiway_protocol::gateway::ModelCallLog;
use aiway_protocol::gateway::request_log::RequestLog;
use aiway_protocol::logg::LogEntry;
use busi::res::{PageRes, Res};
use rocket::serde::json::Json;
use rocket::{State, post, routes};

pub fn routes() -> Vec<rocket::Route> {
    routes![list, request_log_list, model_call_log_list, delete_log]
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

#[post("/list/request-logs", data = "<req>")]
pub async fn request_log_list(
    req: Json<LogListReq>,
    _user: UserPrincipal,
    args: &State<Args>,
) -> Res<PageRes<RequestLog>> {
    match service::request_log_list(req.0, args).await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}

#[post("/list/model-call-logs", data = "<req>")]
pub async fn model_call_log_list(
    req: Json<ModelCallLogListReq>,
    _user: UserPrincipal,
    args: &State<Args>,
) -> Res<PageRes<ModelCallLog>> {
    match service::model_call_log_list(req.0, args).await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}

#[post("/delete/<index>", data = "<req>")]
pub async fn delete_log(
    index: String,
    req: Json<DeleteLogReq>,
    _user: UserPrincipal,
    args: &State<Args>,
) -> Res<()> {
    match service::delete_log(&index, req.0, args).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}
