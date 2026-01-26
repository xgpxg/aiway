use crate::server::auth::UserPrincipal;
use crate::server::node::request::GatewayNodeListReq;
use crate::server::node::response::{GatewayNodeListRes, UsageRes};
use crate::server::node::service;
use busi::res::{PageRes, Res};
use rocket::serde::json::Json;
use rocket::{get, post, routes};

pub fn routes() -> Vec<rocket::Route> {
    routes![
        list,
        node_cpu_usage,
        node_memory_usage,
        node_network_usage,
        node_connection_usage
    ]
}

/// 网关节点列表
#[post("/list", data = "<req>")]
pub async fn list(
    req: Json<GatewayNodeListReq>,
    _user: UserPrincipal,
) -> Res<PageRes<GatewayNodeListRes>> {
    match service::list(req.0).await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// CPU使用率
#[get("/<node_id>/cpu_usage?<start_timestamp>&<end_timestamp>")]
async fn node_cpu_usage(
    node_id: &str,
    start_timestamp: i64,
    end_timestamp: i64,
    _user: UserPrincipal,
) -> Res<Vec<UsageRes>> {
    match service::node_cpu_usage(node_id, start_timestamp, end_timestamp).await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 内存使用量
#[get("/<node_id>/memory_usage?<start_timestamp>&<end_timestamp>")]
async fn node_memory_usage(
    node_id: &str,
    start_timestamp: i64,
    end_timestamp: i64,
    _user: UserPrincipal,
) -> Res<Vec<UsageRes>> {
    match service::node_memory_usage(node_id, start_timestamp, end_timestamp).await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 网络流量
#[get("/<node_id>/network_usage?<start_timestamp>&<end_timestamp>")]
async fn node_network_usage(
    node_id: &str,
    start_timestamp: i64,
    end_timestamp: i64,
    _user: UserPrincipal,
) -> Res<(Vec<UsageRes>, Vec<UsageRes>)> {
    match service::node_network_usage(node_id, start_timestamp, end_timestamp).await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 连接数
#[get("/<node_id>/connection_usage?<start_timestamp>&<end_timestamp>")]
async fn node_connection_usage(
    node_id: &str,
    start_timestamp: i64,
    end_timestamp: i64,
    _user: UserPrincipal,
) -> Res<(Vec<UsageRes>, Vec<UsageRes>, Vec<UsageRes>, Vec<UsageRes>)> {
    match service::node_connection_usage(node_id, start_timestamp, end_timestamp).await {
        Ok(res) => Res::success(res),
        Err(e) => Res::error(&e.to_string()),
    }
}
