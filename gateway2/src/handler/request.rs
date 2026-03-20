//! # 提取请求上下文 - Pingora 实现
//! ## 主要功能
//! 从请求中提出可序列化的请求上下文，包括请求基本信息、body 等数据。
//!
//! ## 基本准则
//! - 在鉴权通过后执行。
//! - 由系统内置，不可关闭。
//! - 提取出的请求信息并缓存，供后续使用。
//! - 不应涉及任何网络请求及 IO 操作，需要在 1ms 内完成。
//! - 上下文应运行在请求流程中被修改。
//!

use crate::handler::{HttpError, HttpResult};
use crate::{get_header, set_header};
use aiway_protocol::context::{HttpContext, SessionExt};
use context::Headers;
use pingora::prelude::*;

pub async fn request_handle(session: &mut Session, context: &mut HttpContext) -> HttpResult<()> {
    let request_id = uuid::Uuid::new_v4().to_string();
    let request_time = chrono::Local::now().timestamp_millis();

    session.set_request_header(Headers::REQUEST_ID, &request_id);
    session.set_request_header(Headers::REQUEST_TIME, &request_time.to_string());

    // let uri = &session.req_header().uri;

    set_header!(session, Headers::REQUEST_ID, request_id);
    set_header!(session, Headers::REQUEST_TIME, request_time.to_string());
    // // 请求ID
    // context.request.request_id = request_id.clone();
    //
    // // 请求时间
    // context.request.request_ts = request_time.clone();

    // 请求路径
    //context.request.path = uri.path().to_string().into();

    // 请求方法
    // context.request.method = session.req_header().method.to_string().into();

    // Host
    // let host = if session.is_http2() {
    //     get_header!(session, ":authority")
    // } else {
    //     get_header!(session, "host")
    // };

    // match host {
    //     None => {
    //         return Err(HttpError::new(500, "host is empty"));
    //     }
    //     Some(host) => {
    //         context.request.host = host.to_string();
    //     }
    // }

    Ok(())
}
