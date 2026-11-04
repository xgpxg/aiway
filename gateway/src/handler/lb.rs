//! # 负载均衡 - Pingora 实现
//!
use crate::components::Servicer;
use crate::handler::{HandlerError, HandlerResult};
use aiway_protocol::context::HttpContext;
use pingora::prelude::*;

// 负载均衡用到的上下文 key，仅负载均衡用到，所以不放在http_context中

/// 候选实例列表在上下文中的 key
pub const LB_CANDIDATES: &str = ":lb:candidates";
/// failover 尝试计数器在上下文中的 key
pub const LB_ATTEMPT: &str = ":lb:attempt";
/// 本次请求已计数的主实例地址（供 cleanup 阶段对称地 -1 连接数）
pub const LB_CONN_GUARD: &str = ":lb:conn_guard";

pub async fn lb_handle(_session: &mut Session, ctx: &mut HttpContext) -> HandlerResult<()> {
    let route = ctx.get_route();
    if route.is_none() {
        return Ok(());
    }

    let route = route.unwrap();
    let service = route.get_service();
    if service.is_empty() {
        log::warn!("No valid service matched for route path: {}", route.path);
        return Err(HandlerError(
            502,
            "No available service matched".to_string(),
        ));
    }

    // 获取客户端真实IP，参与负载均衡决策
    let client_ip = ctx.client_ip();

    // 可用实例
    let instances = Servicer::get_instances(service, client_ip.as_deref());

    if instances.is_empty() {
        log::warn!("No available instance for service: {}", service);
        return Err(HandlerError(
            502,
            format!("No available instance for service: {}", service),
        ));
    }

    // 首个为负载均衡选出的主实例
    let primary = instances[0].clone();
    ctx.set_routing_url(primary.clone());

    // 最少连接策略需要实时在途连接数：主实例 +1，cleanup 阶段 -1
    Servicer::inc_conn(&primary);
    ctx.insert_any_state(LB_CONN_GUARD, primary);

    // 存储候选列表，用于 upstream_peer 的故障自动切换
    ctx.insert_any_state(LB_CANDIDATES, instances);

    Ok(())
}
