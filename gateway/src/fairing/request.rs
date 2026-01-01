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

use crate::context::{HCM, Headers};
use crate::skip_if_error;
use dashmap::DashMap;
use protocol::SV;
use protocol::gateway::{HttpContext, RequestContext, ResponseContext};
use rocket::data::ToByteUnit;
use rocket::fairing::Fairing;
use rocket::{Data, Request};
use std::sync::Arc;

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

    async fn on_request(&self, req: &mut Request<'_>, _data: &mut Data<'_>) {
        skip_if_error!(req);

        // 请求体
        // let body_data = data.peek(100.mebibytes().as_u64() as usize).await;
        // 注意：这里无法取到完整的body数据，因为peek的限制，只能呢取到512个字节。
        // 而open函数有需要self获取所有权，目前rocket的API无法满足
        // 只能放到具体的接口中使用Data来获取，然后设置到context中

        // 注意Key为小写
        let headers = req
            .headers()
            .iter()
            // 移除不需要透传到下游服务的Header
            .filter(|h| h.name().ne("content-length") && h.name().ne("authorization"))
            .map(|h| (h.name().to_string(), h.value().to_string()))
            .collect::<DashMap<String, String>>();

        // 请求ID
        let request_id = req.headers().get_one(Headers::REQUEST_ID).unwrap();
        // 请求时间戳
        let request_time = req.headers().get_one(Headers::REQUEST_TIME).unwrap();

        // 请求上下文
        let request_context = RequestContext {
            request_id: request_id.to_string(),
            request_ts: request_time.parse().unwrap(),
            method: SV::new(req.method().as_str().into()),
            path: SV::new(req.uri().path().to_string()),
            headers,
            query: req
                .query_fields()
                .map(|q| (q.name.to_string(), q.value.to_string()))
                .collect::<DashMap<String, String>>(),
            body: Default::default(),
            state: Default::default(),
            route: SV::empty(),
            routing_url: SV::empty(),
            routing_path: SV::new(req.uri().path().to_string()),
            host: req.host().unwrap().to_string(),
        };

        // 响应上下文
        let response_context = ResponseContext::default();

        let context = HttpContext {
            request: request_context,
            response: response_context,
        };

        HCM.set(request_id, Arc::new(context));
    }
}
