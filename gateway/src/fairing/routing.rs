//! # 路由匹配
//! 通过请求的path匹配路由，并将路由设置到上下文。
//!
use crate::constants;
use crate::context::HCM;
use crate::router::ROUTER;
use rocket::fairing::Fairing;
use rocket::http::Method;
use rocket::http::uri::Origin;
use rocket::request::FromRequest;
use rocket::serde::json::serde_json::json;
use rocket::{Data, Request};

pub struct Routing {}
impl Routing {
    pub fn new() -> Self {
        Self {}
    }
}

#[rocket::async_trait]
impl Fairing for Routing {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "Routing",
            kind: rocket::fairing::Kind::Request,
        }
    }

    async fn on_request(&self, req: &mut Request<'_>, _data: &mut Data<'_>) {
        // 获取path
        // 注意这个path非全路径，而是去除了网关API前缀后的路径
        let path = &req.uri().path().as_str()[constants::API_PREFIX.len()..];
        let route = match ROUTER.matches(path) {
            Some(r) => r,
            None => {
                // 没有匹配到路由，修改uri，转发到502端点
                log::warn!("No route matched for path: {}", path);
                req.set_method(Method::Get);
                req.set_uri(Origin::parse("/eep/502").unwrap());
                return;
            }
        };
        let context = HCM.get_from_request(req);
        context.request.set_route(route);
    }
}
