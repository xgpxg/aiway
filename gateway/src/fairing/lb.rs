//! # 负载均衡
//!
use crate::context::HCM;
use crate::router::Servicer;
use crate::{set_error, skip_if_error};
use rocket::fairing::Fairing;
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
        skip_if_error!(req);
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
            match Servicer::get_instance(service) {
                Some(instance) if !instance.is_empty() => {
                    // 设置最终需要转发的URL
                    context.request.set_routing_url(instance);
                    return;
                }
                _ => {
                    log::warn!("No available instance for service: {}", service);
                }
            }
        }

        set_error!(req, 502, "BadGateway");
    }
}
