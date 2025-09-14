//! # 接口级别的过滤器
//! ## 主要功能
//! 在请求到达接口时，执行业务逻辑前，对请求进行拦截处理，可修改请求参数等。
//!
//! ## 基本准则
//! - 执行API业务逻辑之前执行。
//! - 默认不执行任何过滤器，由用户自行配置
//! - 需要支持自定义脚本执行
use rocket::fairing::Fairing;
use rocket::{Data, Request};

pub struct PreFilter {}
impl PreFilter {
    pub fn new() -> Self {
        Self {}
    }
}

#[rocket::async_trait]
impl Fairing for PreFilter {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "PreFilter",
            kind: rocket::fairing::Kind::Request,
        }
    }

    async fn on_request(&self, _req: &mut Request<'_>, _data: &mut Data<'_>) {
        // 1. 加载全局插件

        // 2. 按顺序执行插件

        //println!("Run PreFilter on request");
    }
}

pub struct PostFilter {}
impl PostFilter {
    pub fn new() -> Self {
        Self {}
    }
}

#[rocket::async_trait]
impl Fairing for PostFilter {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "PostFilter",
            kind: rocket::fairing::Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _req: &'r Request<'_>, _res: &mut rocket::Response<'r>) {
        // 1. 加载全局过滤器

        // 2. 执行过滤器

        //println!("Run PostFilter on response");
    }
}
