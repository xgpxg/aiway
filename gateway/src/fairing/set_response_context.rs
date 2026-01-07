//! # 设置响应上下文
//! 设置响应上下文，提供给插件执行。
//!
//! 如果路由插件和全局插件都没配置，则无需设置。
//!
use crate::components::GLOBAL_FILTER;
use context::{HCM, skip_if_error};
use rocket::Request;
use rocket::fairing::Fairing;

pub struct SetResponseContext {}
impl SetResponseContext {
    pub fn new() -> Self {
        Self {}
    }
}

#[rocket::async_trait]
impl Fairing for SetResponseContext {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "SetResponseContext",
            kind: rocket::fairing::Kind::Response,
        }
    }

    async fn on_response<'r>(&self, req: &'r Request<'_>, res: &mut rocket::Response<'r>) {
        skip_if_error!(req);
        let context = HCM.get_from_request(req);
        let route = context.request.get_route().unwrap();
        let route_plugins = &route.post_filters;

        let config = GLOBAL_FILTER.get().unwrap().config.read().await;
        let global_plugins = &config.post_filters;

        if route_plugins.is_empty() && global_plugins.is_empty() {
            return;
        }

        // 如果插件不为空 设置响应上下文
        let response_context = &context.response;

        // 设置响应状态码
        response_context.set_status(res.status().code);

        // 设置响应头
        response_context.set_headers(
            res.headers()
                .iter()
                .map(|h| (h.name.to_string(), h.value.to_string())),
        );

        // 设置响应体
        response_context.set_body(res.body_mut().to_bytes().await.unwrap_or_default());

        //TODO stream处理
    }
}
