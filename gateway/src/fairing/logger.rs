//! # 日志记录
//!
//! 在请求结束前记录日志，理论上该fairing总是需要被调用
//!
//! # 日志存储实现方案
//! 集群环境下使用quickwit，单机环境下使用tantivy。
//!
//! - 网关收集每次请求的日志，达到阈值或时间后批量发送到日志服务。
//! - 网关本身不应该对日志做任何处理，只做收集和发送，避免影响性能。
//!

use crate::context::{HCM, Headers};
use crate::{Args, skip_if_error};
use clap::Parser;
use protocol::gateway::request_log::RequestLog;
use rocket::Request;
use rocket::fairing::Fairing;
use tokio::sync::mpsc;

pub struct Logger {
    args: Args,
}
impl Logger {
    pub fn new() -> Self {
        Self {
            args: Args::parse(),
        }
    }
}

#[rocket::async_trait]
impl Fairing for Logger {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "Logger",
            kind: rocket::fairing::Kind::Request | rocket::fairing::Kind::Response,
        }
    }

    async fn on_response<'r>(&self, req: &'r Request<'_>, res: &mut rocket::Response<'r>) {
        // 注意：由于请求上下文构建是在安全校验之后的，如果安全校验不通过，则获取不到上下文，因此不能通过上下文提取日志。
        //skip_if_error!(req);
        // 提取RequestContext
        // let context = HCM.get_from_request(&req);
        //
        // let req_cxt = &context.request;
        // let res_cxt = &context.response;

        let client_ip = req.client_ip().unwrap();

        // 请求时间戳
        let request_time = req
            .headers()
            .get_one(Headers::REQUEST_TIME)
            .unwrap()
            .parse::<i64>()
            .unwrap();
        let request_id = req.headers().get_one(Headers::REQUEST_ID).unwrap();

        let response_time = chrono::Local::now().timestamp_millis();

        let request_log = RequestLog {
            request_id: request_id.to_string(),
            client_ip: client_ip.to_string(),
            client_country: None,
            client_province: None,
            client_city: None,
            method: req.method().to_string(),
            path: req.uri().path().to_string(),
            request_time,
            response_time,
            elapsed: response_time - request_time,
            status_code: res.status().code,
            response_size: res.body().preset_size(),
            user_agent: req
                .headers()
                .get_one(Headers::USER_AGENT)
                .map(|s| s.to_string()),
            referer: req
                .headers()
                .get_one(Headers::REFERER)
                .map(|s| s.to_string()),
            node_address: format!("{}:{}", self.args.address, self.args.port),
        };

        match serde_json::to_vec(&request_log) {
            Ok(value) => logging::log_request(value),
            Err(e) => log::error!("Failed to serialize RequestLog to JSON: {}", e),
        }
    }
}
