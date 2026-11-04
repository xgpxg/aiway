//! # 清理工作
//!
use crate::components::Servicer;
use crate::handler::lb::LB_CONN_GUARD;
use crate::report::STATE;
use aiway_protocol::context::HttpContext;
use pingora::prelude::*;

pub async fn cleanup_handle(_session: &mut Session, ctx: &HttpContext) {
    STATE.inc_http_connect_count(-1);
    if ctx.is_sse() {
        STATE.inc_sse_connect_count(-1);
    }
    if ctx.is_websocket() {
        STATE.inc_websocket_connect_count(-1);
    }

    // 释放负载均衡主实例的在途连接计数（供 LeastConn 使用）
    if let Some(addr) = ctx.get_any_state::<String>(LB_CONN_GUARD) {
        Servicer::dec_conn(&addr);
    }
}
