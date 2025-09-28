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
//! ## 过滤器加载
//! 1. 从控制台加载全局的网关配置
//! 2. 获取过滤器
//! 3. 按顺序执行
//!

use crate::context::HCM;
use crate::router::{GATEWAY_CONFIG, PLUGINS};
use rocket::fairing::Fairing;
use rocket::http::Method;
use rocket::http::uri::Origin;
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

    async fn on_request(&self, req: &mut Request<'_>, _data: &mut Data<'_>) {
        let _ = crate::extract_api_path!(req);

        let context = HCM.get_from_request(req);
        let config = GATEWAY_CONFIG.get().unwrap().config.read().await;
        let plugins = &config.pre_filters;

        for configured_plugin in plugins.iter() {
            log::debug!(
                "execute global post filter plugin: {}",
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
                    req.set_method(Method::Get);
                    req.set_uri(Origin::parse("/eep/502").unwrap());
                    return;
                }
            }
        }
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
        let context = HCM.get_from_request(req);
        let config = GATEWAY_CONFIG.get().unwrap().config.read().await;
        let plugins = &config.post_filters;

        for configured_plugin in plugins.iter() {
            log::debug!(
                "execute global post filter plugin: {}",
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
                        "execute global post filter plugin {} error: {}",
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
