//! # 路由级别的过滤器
//! ## 主要功能
//! 在请求即将到达接口前，对请求进行拦截处理，可修改请求参数以及自定义逻辑。
//!
//! ## 基本准则
//! - 执行API业务逻辑之前执行。
//! - 默认不执行任何过滤器，由用户自行配置
//! - 需要支持自定义脚本执行
//!
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

    /// 请求阶段过滤器实现
    ///
    /// - 支持多个过滤器串联执行
    /// - 默认不执行任何过滤器，由用户自行配置
    /// - 可在此处修改请求参数
    async fn on_request(&self, req: &mut Request<'_>, _data: &mut Data<'_>) {
        // 1. 获取path，匹配路由
        let path = req.uri().path().as_str();

        // 2. 加载插件

        // 3. 按顺序执行插件

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
        // 1. 获取path，匹配路由
        let path = req.uri().path().as_str();

        // 2. 加载插件

        // 3. 按顺序执行插件

        //println!("path: {}", context.get_path());
        //println!("Run PostFilter on response");
    }
}
