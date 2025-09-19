//! # 负载均衡
//!
use crate::context::HCM;
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
        let context = HCM.get_from_request(req);
        let route = context.request.get_route();
        if route.is_none() {
            return;
        }
    }
}
