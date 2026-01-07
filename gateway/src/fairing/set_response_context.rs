//! # 设置响应数据
//! ## 主要功能
//! 在响应客户端前，将上下文中的响应数据附加到响应中。
//!
//! ## 基本准则
//! - 该fairing必须执行
//! - 使用覆盖模式，即上下文中的响应数据优先覆盖原始响应中的数据。这是因为，上下文中的数据可能是由插件修改而来，应该优先被设置。
//!
use crate::components::GLOBAL_FILTER;
use context::{skip_if_error, HCM};
use rocket::fairing::Fairing;
use rocket::Request;

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

        response_context.set_status(res.status().code);
        response_context.set_headers(
            res.headers()
                .iter()
                .map(|h| (h.name.to_string(), h.value.to_string())),
        );
        response_context.set_body(res.body_mut().to_bytes().await.unwrap_or_default());
    }
}
