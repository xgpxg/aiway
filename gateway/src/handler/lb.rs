//! # 负载均衡 - Pingora 实现
//!
use crate::components::Servicer;
use crate::handler::{HandlerError, HandlerResult};
use aiway_protocol::context::HttpContext;
use pingora::prelude::*;

/// 候选实例列表在上下文中的 key
pub const LB_CANDIDATES: &str = ":lb:candidates";
/// failover 尝试计数器在上下文中的 key
pub const LB_ATTEMPT: &str = ":lb:attempt";

pub async fn lb_handle(_: &mut Session, ctx: &mut HttpContext) -> HandlerResult<()> {
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

    let instances = Servicer::get_instances(service);
    
    if instances.is_empty() {
        log::warn!("No available instance for service: {}", service);
        return Err(HandlerError(
            502,
            format!("No available instance for service: {}", service),
        ));
    }

    // 首个为负载均衡选出的主实例
    let primary = instances[0].clone();
    ctx.set_routing_url(primary);

    // 存储候选列表，用于 upstream_peer 的故障自动切换
    ctx.insert_any_state(LB_CANDIDATES, instances);

    Ok(())
}
