//! # 提取请求上下文
//! ## 主要功能
//! 从请求中提出可序列化的请求上下文，包括请求基本信息、body等数据。
//!
//! ## 基本准则
//! - 在鉴权通过后执行。
//! - 由系统内置，不可关闭。
//! - 提取出的请求信息并缓存，供后续使用。
//! - 不应涉及任何网络请求及IO操作，需要在1ms内完成。
//! - 上下文应运行在请求流程中被修改。
//!

use crate::context::HCM;
use protocol::SV;
use protocol::gateway::{HttpContext, RequestContext, ResponseContext};
use rocket::data::ToByteUnit;
use rocket::fairing::Fairing;
use rocket::http::Header;
use rocket::{Data, Request};
use std::sync::Arc;
use uuid::Uuid;
use crate::skip_if_error;

pub struct RequestData {}
impl RequestData {
    pub fn new() -> Self {
        Self {}
    }
}

#[rocket::async_trait]
impl Fairing for RequestData {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "RequestData",
            kind: rocket::fairing::Kind::Request,
        }
    }

    async fn on_request(&self, req: &mut Request<'_>, data: &mut Data<'_>) {
        skip_if_error!(req);
        // 请求ID，应仅在此处生成一次，后续通过该ID获取上下文
        let request_id = Uuid::new_v4().to_string();

        // 请求体
        let body_data = data.peek(100.mebibytes().as_u64() as usize).await;

        // 请求上下文
        let request_context = RequestContext {
            request_id: request_id.clone(),
            request_ts: chrono::Local::now().timestamp_millis(),
            method: SV::new(req.method().as_str().into()),
            path: SV::new(req.uri().path().to_string()),
            headers: Default::default(),
            query: SV::new(req.uri().query().map(|v| v.to_string())),
            body: SV::new(body_data.to_vec()),
            state: Default::default(),
            route: SV::empty(),
            routing_url: SV::empty(),
        };

        // 响应上下文
        let response_context = ResponseContext::default();

        let context = HttpContext {
            request: request_context,
            response: response_context,
        };

        HCM.set(&request_id, Arc::new(context));

        // 添加请求ID，用于后续获取上下文
        req.add_header(Header::new(crate::context::Headers::REQUEST_ID, request_id));
    }
}
