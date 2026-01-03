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

use context::HttpContextFairing;
use rocket::fairing::Fairing;
use rocket::{Data, Request};

pub struct RequestData {
    http_context_fairing: HttpContextFairing,
}
impl RequestData {
    pub fn new() -> Self {
        Self {
            http_context_fairing: HttpContextFairing,
        }
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
        self.http_context_fairing.on_request(req, data).await;
    }
}
