//! # 负载均衡
//!
use crate::context::HCM;
use crate::router::SERVICES;
use rocket::fairing::Fairing;
use rocket::http::Method;
use rocket::http::uri::Origin;
use rocket::{Data, Request};

pub struct LoadBalance {}
impl LoadBalance {
    pub fn new() -> Self {
        Self {}
    }
}

#[rocket::async_trait]
impl Fairing for LoadBalance {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "LoadBalance",
            kind: rocket::fairing::Kind::Request,
        }
    }

    async fn on_request(&self, req: &mut Request<'_>, _data: &mut Data<'_>) {
        let context = HCM.get_from_request(req);
        let route = context.request.get_route();
        if route.is_none() {
            return;
        }

        let route = route.unwrap();
        let service = route.get_service();
        if service.is_empty() {
            // 没有匹配到service或service为空，修改uri，转发到502端点
            log::warn!("No valid service matched for route path: {}", route.path);
        } else {
            match SERVICES.get().unwrap().get_instance(service) {
                Some(instance) if !instance.is_empty() => {
                    // 设置最终需要转发的URL
                    context.request.set_routing_url(instance);
                    return;
                }
                _ => {
                    log::warn!("No valid instance available for service: {}", service);
                }
            }
        }
        req.set_method(Method::Get);
        req.set_uri(Origin::parse("/eep/502").unwrap());
    }
}
