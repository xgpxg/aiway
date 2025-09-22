//! # 鉴权
//! ## 主要功能
//! 从请求中提取ApiKey并验证，验证不通过则返回403。
//!
//! 考虑是调用另外的服务验证，还是对API Key解密验证?
//!
use crate::context::Headers;
use protocol::gateway::ApiKey;
use rocket::fairing::Fairing;
use rocket::http::Method;
use rocket::http::uri::Origin;
use rocket::{Data, Request};

pub struct Authentication {}
impl Authentication {
    pub fn new() -> Self {
        Self {}
    }
}

const BEARER_PREFIX: &str = "Bearer ";

#[rocket::async_trait]
impl Fairing for Authentication {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "Authentication",
            kind: rocket::fairing::Kind::Request,
        }
    }

    async fn on_request(&self, req: &mut Request<'_>, _data: &mut Data<'_>) {
        let _ = crate::extract_api_path!(req);

        let bearer_token = req.headers().get_one(Headers::AUTHORIZATION);
        let api_key = match bearer_token {
            Some(api_key) => match api_key.strip_prefix(BEARER_PREFIX) {
                Some(api_key) => api_key,
                None => {
                    req.set_method(Method::Get);
                    req.set_uri(Origin::parse("/eep/401").unwrap());
                    return;
                }
            },
            None => {
                req.set_method(Method::Get);
                req.set_uri(Origin::parse("/eep/401").unwrap());
                return;
            }
        };

        if let Err(_) = ApiKey::decrypt(&[0; 32], api_key) {
            req.set_method(Method::Get);
            req.set_uri(Origin::parse("/eep/401").unwrap());
            return;
        }

        let api_key = cache::get::<String>(api_key).await.unwrap();
        let _api_key = match api_key {
            None => {
                req.set_method(Method::Get);
                req.set_uri(Origin::parse("/eep/401").unwrap());
                return;
            }
            Some(v) => v,
        };

        //println!("{:?}", api_key.unwrap().principal);
    }
}
