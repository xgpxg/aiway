//! # 鉴权
//! ## 主要功能
//! 从请求中提取ApiKey并验证，验证不通过则返回403。
//!
//! 考虑是调用另外的服务验证，还是对API Key解密验证?
//!
use crate::context::Headers;
use crate::{set_error, skip_if_error};
use cache::caches::CacheKey;
use common::constants::ENCRYPT_KEY;
use protocol::gateway::ApiKey;
use rocket::fairing::Fairing;
use rocket::{Data, Request};
use serde_json::Value;

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
        skip_if_error!(req);

        let bearer_token = req.headers().get_one(Headers::AUTHORIZATION);

        let api_key = match bearer_token {
            Some(api_key) => match api_key.strip_prefix(BEARER_PREFIX) {
                Some(api_key) => api_key,
                None => {
                    set_error!(req, 401, "Unauthorized");
                    return;
                }
            },
            None => {
                set_error!(req, 401, "Unauthorized");
                return;
            }
        };

        if let Err(_) = ApiKey::decrypt(ENCRYPT_KEY, api_key) {
            set_error!(req, 401, "Unauthorized");
            return;
        }

        let api_key = cache::get::<Value>(&CacheKey::ApiKey(api_key.to_string()).to_string())
            .await
            .unwrap();
        if api_key.is_none() {
            set_error!(req, 401, "Unauthorized");
            return;
        }
    }
}
