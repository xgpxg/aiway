//! # 日志记录
//!
use crate::components::IpRegion;
use crate::handler::HttpResult;
use aiway_protocol::context::{HttpContext, SessionExt};
use aiway_protocol::gateway::request_log::RequestLog;
use pingora::prelude::*;
use aiway_protocol::common::header::Headers;

pub async fn log_handle(session: &Session, resp: &mut ResponseHeader, context: &HttpContext) {
    // 记录请求日志
    let client_ip = session.client_addr().unwrap().to_string();

    // 请求ID
    let request_id = session.get_request_header(Headers::REQUEST_ID).unwrap();

    // 请求时间戳
    let request_time = session
        .get_request_header(Headers::REQUEST_TIME)
        .unwrap()
        .parse::<i64>()
        .unwrap();

    // 响应时间戳
    let response_time = chrono::Local::now().timestamp_millis();

    let method = &session.req_header().method;
    let path = &session.req_header().uri.path();
    let status_code = &resp.status;
    let ua = session.get_request_header(Headers::USER_AGENT);
    let refer = session.get_request_header(Headers::REFERER);

    // 地理位置
    let region = IpRegion::search(&client_ip);

    let request_log = RequestLog {
        request_id: request_id.to_string(),
        client_ip: client_ip.to_string(),
        client_country: region.0,
        client_province: region.1,
        client_city: region.2,
        method: method.to_string(),
        path: path.to_string(),
        request_time,
        response_time,
        elapsed: response_time - request_time,
        status_code: status_code.as_u16(),
        //TODO
        response_size: Some(0),
        user_agent: ua.map(|s| s.to_string()),
        referer: refer.map(|s| s.to_string()),
        // TODO
        node_address: "".to_string(),
    };

    match serde_json::to_vec(&request_log) {
        Ok(value) => logging::log_request(value),
        Err(e) => log::error!("Failed to serialize RequestLog to JSON: {}", e),
    }
}
