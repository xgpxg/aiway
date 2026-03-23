//! # 清理工作
//!
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
}
