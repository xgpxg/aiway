//! # 预处理
//!
use crate::handler::HttpResult;
use crate::report::STATE;
use aiway_protocol::context::{HttpContext};
use pingora::prelude::*;

pub async fn pre_handle(_: &mut Session, _: &mut HttpContext) -> HttpResult<()> {
    // 请求计数（含所有请求，只要网关收到请求，就计数）
    STATE.inc_request_count(1);

    Ok(())
}
