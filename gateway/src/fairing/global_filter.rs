//! # 全局过滤器
//! ## 主要功能
//! 对请求/响应进行拦截处理，对整个网关有效。
//!
//! ## 基本准则
//! - 在提请求数据后执行。
//! - 可由用户自由配置，串联执行
//! - 要能够支持执行脚本
//! - 可能涉及到网络请求，需考虑性能
//! - 系统可能内置一些过滤器，但也可以由用户自定义实现。
//!
//! 注意：该过滤器全局有效，针对每个API的过滤器需使用`PreFilter`
//!
use crate::context::Headers;
use rocket::fairing::Fairing;
use rocket::http::Header;
use rocket::{Data, Request};

pub struct GlobalPreFilter {}
impl GlobalPreFilter {
    pub fn new() -> Self {
        Self {}
    }
}

#[rocket::async_trait]
impl Fairing for GlobalPreFilter {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "GlobalPreFilter",
            kind: rocket::fairing::Kind::Request,
        }
    }

    async fn on_request(&self, _req: &mut Request<'_>, _data: &mut Data<'_>) {
        // 1. 加载全局插件

        // 2. 按顺序执行插件

        //println!("Run GlobalPreFilter on request");
    }
}

pub struct GlobalPostFilter {}
impl GlobalPostFilter {
    pub fn new() -> Self {
        Self {}
    }
}

#[rocket::async_trait]
impl Fairing for GlobalPostFilter {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "GlobalPostFilter",
            kind: rocket::fairing::Kind::Response,
        }
    }

    async fn on_response<'r>(&self, req: &'r Request<'_>, res: &mut rocket::Response<'r>) {
        // 添加请求ID
        res.set_header(Header::new(
            Headers::REQUEST_ID,
            Headers::get_request_id(req),
        ));
        // 1. 加载全局插件

        // 2. 执行过滤器

        //println!("Run GlobalPostFilter on response");
    }
}
