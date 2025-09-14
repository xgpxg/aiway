//! # 接口级别的过滤器
//! ## 主要功能
//! 在请求即将到达接口前，对请求进行拦截处理，可修改请求参数以及自定义逻辑。
//!
//! ## 基本准则
//! - 执行API业务逻辑之前执行。
//! - 默认不执行任何过滤器，由用户自行配置
//! - 需要支持自定义脚本执行
//!
use crate::context::RCM;
use rocket::fairing::Fairing;
use rocket::http::Header;
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

    /// 请求阶段过滤器实现
    ///
    /// - 支持多个过滤器串联执行
    /// - 默认不执行任何过滤器，由用户自行配置
    /// - 可在此处修改请求参数
    async fn on_request(&self, req: &mut Request<'_>, _data: &mut Data<'_>) {
        // 1. 加载插件

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

    async fn on_response<'r>(&self, req: &'r Request<'_>, res: &mut rocket::Response<'r>) {
        // 1. 加载自定义过滤器

        // 2. 执行过滤器

        let context = RCM.get_from_request(&req);
        context.set_header("X-AAA", "123");

        res.set_header(Header::new("X-AAA", context.get_header("X-AAA").unwrap()));

        //println!("path: {}", context.get_path());
        //println!("Run PostFilter on response");
    }
}
