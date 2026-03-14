use crate::server::auth::UserPrincipal;
use crate::server::mcp::request::{
    McpServerAddReq, McpServerStatusUpdateReq, McpServerUpdateReq, McpToolAddReq, McpToolUpdateReq,
    UpdateMcpToolStatusReq,
};
use crate::server::mcp::response::{McpServerListRes, McpToolListRes};
use crate::server::mcp::{McpServerListReq, McpToolListReq, service};
use busi::req::IdsReq;
use busi::res::{PageRes, Res};
use rocket::serde::json::Json;
use rocket::{post, routes};

pub fn routes() -> Vec<rocket::Route> {
    routes![
        server_add,
        server_list,
        server_update,
        server_update_status,
        server_delete,
        tool_add,
        tool_list,
        tool_update,
        tool_delete,
        tool_update_status,
    ]
}

#[post("/server/add", data = "<req>")]
async fn server_add(req: Json<McpServerAddReq>, user: UserPrincipal) -> Res<()> {
    match service::server_add(req.0, user).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

#[post("/server/list", data = "<req>")]
async fn server_list(
    req: Json<McpServerListReq>,
    _user: UserPrincipal,
) -> Res<Vec<McpServerListRes>> {
    match service::server_list(req.0).await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}

#[post("/server/update", data = "<req>")]
async fn server_update(req: Json<McpServerUpdateReq>, user: UserPrincipal) -> Res<()> {
    match service::server_update(req.0, user).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

#[post("/server/update_status", data = "<req>")]
async fn server_update_status(req: Json<McpServerStatusUpdateReq>, user: UserPrincipal) -> Res<()> {
    match service::server_update_status(req.0, user).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

#[post("/server/delete", data = "<req>")]
async fn server_delete(req: Json<IdsReq>, _user: UserPrincipal) -> Res<()> {
    match service::server_delete(req.0).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

#[post("/tool/add", data = "<req>")]
async fn tool_add(req: Json<McpToolAddReq>, user: UserPrincipal) -> Res<()> {
    match service::tool_add(req.0, user).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

#[post("/tool/list", data = "<req>")]
async fn tool_list(
    req: Json<McpToolListReq>,
    _user: UserPrincipal,
) -> Res<PageRes<McpToolListRes>> {
    match service::tool_list(req.0).await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}

#[post("/tool/update", data = "<req>")]
async fn tool_update(req: Json<McpToolUpdateReq>, user: UserPrincipal) -> Res<()> {
    match service::tool_update(req.0, user).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

#[post("/tool/delete", data = "<req>")]
async fn tool_delete(req: Json<IdsReq>, _user: UserPrincipal) -> Res<()> {
    match service::tool_delete(req.0).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

#[post("/tool/update_status", data = "<req>")]
async fn tool_update_status(req: Json<UpdateMcpToolStatusReq>, user: UserPrincipal) -> Res<()> {
    match service::tool_update_status(req.0, user).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}
