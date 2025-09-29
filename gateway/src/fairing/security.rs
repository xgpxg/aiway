//! # 安全验证
//! ## 主要功能
//! 对原始请求进行基本的安全验证。
//! ## 基本准则
//! - 由网关系统内置，可通过系统级别的配置开启或关闭。
//! - 需最先执行，以拦截恶意请求。
//! - 不应提取请求body数据，仅对请求url（含query参数）、header等基础数据进行验证。
//! - 当验证失败时，更改uri到指定端点，返回错误信息。
//! - 不应涉及任何网络请求及IO操作，需要在5ms内完成
//!
use crate::report::STATE;
use rocket::fairing::Fairing;
use rocket::{Data, Request};

pub struct PreSecurity {}
impl PreSecurity {
    pub fn new() -> Self {
        Self {}
    }
}

#[rocket::async_trait]
impl Fairing for PreSecurity {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "PreSecurity",
            kind: rocket::fairing::Kind::Request,
        }
    }

    async fn on_request(&self, _req: &mut Request<'_>, _data: &mut Data<'_>) {
        // 请求计数
        STATE.inc_request_count(1);
        // http连接计数
        // 该计数会在cleaner以及panic hook中-1
        STATE.inc_http_connect_count(1);

        //println!("Run PreSecurity on request");
    }
}
