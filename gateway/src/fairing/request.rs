//! # 提取请求上下文
//! ## 主要功能
//! 从请求中提出可序列化的请求上下文，包括请求基本信息、body等数据。
//!
//! ## 基本准则
//! - 在鉴权通过后执行。
//! - 由系统内置，不可关闭。
//! - 因将提取出的请求信息缓存，供后续使用。
//! - 不应涉及任何网络请求及IO操作，需要在1ms内完成。
//! - 上下文应运行在请求流程中被修改。
//!

use crate::context::RCM;
use dashmap::{DashMap, DashSet};
use protocol::gateway::RequestContext;
use rocket::data::ToByteUnit;
use rocket::fairing::Fairing;
use rocket::http::Header;
use rocket::{Data, Request};
use std::sync::Arc;
use uuid::Uuid;

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
        let request_id = Uuid::new_v4().to_string();

        let body_data = data.peek(100.mebibytes().as_u64() as usize).await;

        let context = RequestContext {
            request_id: request_id.clone(),
            method: req.method().as_str().into(),
            path: DashSet::from_iter(vec![req.uri().path().to_string()]),
            headers: Default::default(),
            query: req
                .uri()
                .query()
                .map(|v| DashSet::from_iter(vec![v.to_string()]))
                .unwrap_or_default(),
            body: DashSet::from_iter(vec![body_data.into()]),
            state: Default::default(),
        };

        RCM.set(&request_id, Arc::new(context));

        req.add_header(Header::new(crate::context::Headers::REQUEST_ID, request_id));
    }
}
