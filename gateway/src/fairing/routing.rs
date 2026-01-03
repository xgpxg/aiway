//! # 路由匹配
//! 通过请求的path匹配路由，并将路由设置到上下文。
//!
use crate::components::ROUTER;
use rocket::fairing::Fairing;
use rocket::{Data, Request};
use context::{set_error, skip_if_error, HCM};

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
        skip_if_error!(req);
        let context = HCM.get_from_request(req);
        let route = match ROUTER.get().unwrap().matches(context.clone()) {
            Some(r) => r,
            None => {
                // 没有匹配到路由，返回404
                set_error!(req, 404, "NotFound");
                return;
            }
        };
        context.request.set_route(route);
    }
}
