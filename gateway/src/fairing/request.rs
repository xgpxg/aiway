//! # 提取请求数据
//! ## 主要功能
//! 从请求中提出可序列化的请求数据，包括请求基本信息、body等数据。
//!
//! ## 基本准则
//! - 在鉴权通过后执行。
//! - 由系统内置，不可关闭。
//! - 因将提取出的请求信息缓存，供后续使用。
//! - 不应涉及任何网络请求及IO操作，需要在1ms内完成
//!
use rocket::fairing::Fairing;
use rocket::{Data, Request};

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

    async fn on_request(&self, _req: &mut Request<'_>, _data: &mut Data<'_>) {
        //println!("Run RequestData on request");
    }
}
