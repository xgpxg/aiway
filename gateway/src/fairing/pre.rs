//! # 预处理
//!
use crate::report::STATE;
use rocket::fairing::Fairing;
use rocket::http::Header;
use rocket::{Data, Request};
use uuid::Uuid;
use context::Headers;

pub struct Pre {}
impl Pre {
    pub fn new() -> Self {
        Self {}
    }
}

#[rocket::async_trait]
impl Fairing for Pre {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "Pre",
            kind: rocket::fairing::Kind::Request | rocket::fairing::Kind::Response,
        }
    }

    async fn on_request(&self, req: &mut Request<'_>, _data: &mut Data<'_>) {
        // 请求计数（含所有请求，只要网关收到请求，就计数）
        STATE.inc_request_count(1);

        // 请求ID，应仅在此处生成一次，后续通过该ID获取上下文
        let request_id = Uuid::new_v4().to_string();

        // 添加请求ID，用于后续获取上下文
        req.add_header(Header::new(Headers::REQUEST_ID, request_id));

        // 添加请求时间Header
        req.add_header(Header::new(
            Headers::REQUEST_TIME,
            chrono::Local::now().timestamp_millis().to_string(),
        ));
    }

    async fn on_response<'r>(&self, _req: &'r Request<'_>, res: &mut rocket::Response<'r>) {
        if let Some(content_type) = res.headers().get_one(Headers::CONTENT_TYPE)
            && content_type.starts_with("text/event-stream")
        {
            // 增加SSE连接数
            // 当SSE流结束后，会在openapi中减去
            STATE.inc_sse_connect_count(1);
        }
    }
}
