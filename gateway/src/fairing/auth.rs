//! # 鉴权
//! ## 主要功能
//! 从请求中提取ApiKey并验证，验证不通过则返回403。
//! 验证通过后
//!
//! ## 基本准则
//! - 在基本安全验证后执行。
//! - 由系统内置，不可关闭。
//! - 不应提取请求body数据，仅对请求url（含query参数）、header等基础数据进行验证。
//! - 当验证失败时，更改uri到指定端点，返回错误信息。
//! - 不应涉及任何网络请求及IO操作，需要在5ms内完成
//!
use rocket::fairing::Fairing;
use rocket::{Data, Request};

pub struct Authentication {}
impl Authentication {
    pub fn new() -> Self {
        Self {}
    }
}

#[rocket::async_trait]
impl Fairing for Authentication {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "Authentication",
            kind: rocket::fairing::Kind::Request,
        }
    }

    async fn on_request(&self, _req: &mut Request<'_>, _data: &mut Data<'_>) {
        //println!("Run Authentication on request");
    }
}
