//! # 日志记录
//!
use crate::Args;
use crate::components::IpRegion;
use crate::report::STATE;
use aiway_protocol::common::header::Headers;
use aiway_protocol::context::HttpContext;
use aiway_protocol::context::parts::SerdeParts;
use aiway_protocol::gateway::request_log::RequestLog;
use alert::Alert;
use pingora::prelude::*;

pub async fn log_handle(session: &Session, err: Option<&Error>, ctx: &HttpContext, args: &Args) {
    // SAFE: client_ip不会为空
    let client_ip = session.client_addr().unwrap().to_string();

    let request_id = ctx.request_id();

    let request_time = ctx.request_ts();

    let response_time = chrono::Local::now().timestamp_millis();

    STATE.inc_response_time((response_time - request_time) as usize);

    let request_parts = ctx.request_raw_parts();

    // request_parts 为空的，表明early_request_filter拦截了或者处理失败
    // 此时不记录日志
    if request_parts.is_none() {
        return;
    }

    let request_parts = request_parts.unwrap();

    let method = request_parts
        .method
        .clone()
        .map(|m| m.to_string())
        .unwrap_or_default();

    let request_headers = &request_parts.headers.as_ref();

    let host = request_headers
        .and_then(|h| h.get("host").or_else(|| h.get(":authority")))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let uri = request_parts.uri.as_ref();

    let path = uri.map(|u| u.path().to_string()).unwrap_or_default();

    let ua = request_headers
        .and_then(|h| h.get(Headers::USER_AGENT))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let refer = request_headers
        .and_then(|h| h.get(Headers::REFERER))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let origin = request_headers
        .and_then(|h| h.get(Headers::ORIGIN))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let response_parts = ctx.get_state::<SerdeParts>(HttpContext::RESPONSE_SERDE_PARTS);

    let status_code = response_parts
        .as_ref()
        .and_then(|parts| parts.status_code.map(|s| s.as_u16()));

    let body_size = ctx.get_state::<usize>(HttpContext::RESPONSE_BODY_SIZE);

    let content_type = response_parts
        .as_ref()
        .and_then(|p| p.headers.as_ref())
        .and_then(|h| h.get(Headers::CONTENT_TYPE))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let region = IpRegion::search(&client_ip);

    let request_log = RequestLog {
        request_id: request_id.to_string(),
        client_ip: client_ip.to_string(),
        client_country: region.0,
        client_province: region.1,
        client_city: region.2,
        method,
        path,
        host,
        origin,
        request_parts: serde_json::to_string(&request_parts).unwrap().into(),
        request_time,
        response_time,
        elapsed: response_time - request_time,
        status_code,
        response_size: body_size,
        response_parts: serde_json::to_string(&response_parts).unwrap().into(),
        content_type,
        user_agent: ua.map(|s| s.to_string()),
        referer: refer.map(|s| s.to_string()),
        node_address: format!("{}:{}", args.address, args.port),
    };
    match serde_json::to_vec(&request_log) {
        Ok(value) => logging::log_request(value),
        Err(e) => log::error!("Failed to serialize RequestLog to JSON: {}", e),
    }
}
