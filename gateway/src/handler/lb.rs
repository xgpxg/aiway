//! # 负载均衡 - Pingora 实现
//!
use crate::components::Servicer;
use crate::handler::HandlerResult;
use aiway_protocol::context::HttpContext;
use pingora::prelude::*;

pub async fn lb_handle(_: &mut Session, ctx: &mut HttpContext) -> HandlerResult<()> {
    let route = ctx.get_route();
    if route.is_none() {
        return Ok(());
    }

    let route = route.unwrap();
    let service = route.get_service();
    if service.is_empty() {
        // 没有匹配到service或service为空，修改uri，转发到502端点
        log::warn!("No valid service matched for route path: {}", route.path);
    } else {
        match Servicer::get_instance(service) {
            Some(instance) if !instance.is_empty() => {
                // 设置最终需要转发的URL
                ctx.set_routing_url(instance);
                return Ok(());
            }
            _ => {
                log::warn!("No available instance for service: {}", service);
            }
        }
    }

    Ok(())
}
