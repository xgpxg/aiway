//! # 清理工作
//!
use crate::report::STATE;
use aiway_protocol::context::HttpContext;
use pingora::prelude::*;

pub async fn cleanup_handle(_session: &mut Session, _context: &HttpContext) {
    STATE.inc_http_connect_count(-1);
}
