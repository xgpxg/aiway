//! # 路由级别的过滤器
//! ## 主要功能
//! 在请求即将到达接口前，对请求进行拦截处理，可修改请求参数以及自定义逻辑。
//!
//! ## 基本准则
//! - 执行API业务逻辑之前执行。
//! - 默认不执行任何过滤器，由用户自行配置
//! - 需要支持自定义脚本执行
//!
use crate::context::HCM;
use crate::components::PLUGINS;
use crate::{set_error, skip_if_error};
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
        skip_if_error!(req);
        let context = HCM.get_from_request(req);
        // SAFE：在routing时已经设置
        let route = context.request.get_route().unwrap();
        let pre_filters = &route.pre_filters;
        for configured_plugin in pre_filters.iter() {
            log::debug!(
                "execute route pre filter plugin: {}",
                configured_plugin.name
            );
            let result = PLUGINS
                .get()
                .unwrap() // SAFE: 在启动时已经初始化
                .execute(configured_plugin, context.as_ref())
                .await;
            match result {
                Ok(_) => {}
                Err(e) => {
                    log::error!(
                        "execute global pre filter plugin {} error: {}",
                        configured_plugin.name,
                        e
                    );
                    set_error!(req, 502, "BadGateway");
                    // req.set_method(Method::Get);
                    // req.set_uri(Origin::parse("/eep/502").unwrap());
                    return;
                }
            }
        }
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
        skip_if_error!(req);

        let context = HCM.get_from_request(req);

        let route = context.request.get_route().unwrap();
        let plugins = &route.post_filters;

        for configured_plugin in plugins.iter() {
            log::debug!(
                "execute route post filter plugin: {}",
                configured_plugin.name
            );
            let result = PLUGINS
                .get()
                .unwrap() // SAFE: 在启动时已经初始化
                .execute(configured_plugin, context.as_ref())
                .await;
            match result {
                Ok(_) => {}
                Err(e) => {
                    log::error!(
                        "execute route post filter plugin {} error: {}",
                        configured_plugin.name,
                        e
                    );
                    res.set_status(rocket::http::Status::InternalServerError);
                    return;
                }
            }
        }
    }
}
