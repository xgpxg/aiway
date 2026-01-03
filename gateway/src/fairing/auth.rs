//! # 鉴权
//! ## 主要功能
//! 从请求中提取ApiKey并验证，验证不通过则返回403。
//!
//! 考虑是调用另外的服务验证，还是对API Key解密验证?
//!
use cache::caches::CacheKey;
use common::constants::ENCRYPT_KEY;
use protocol::gateway::ApiKey;
use rocket::fairing::Fairing;
use rocket::{Data, Request};
use serde_json::Value;
use context::{set_error, skip_if_error, Headers, HCM};

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

        // 获取上下文
        let ctx = HCM.get_from_request(req);
        // SAFE: 此时路由一定存在
        let route = ctx.request.get_route().unwrap();

        // 未开启权限验证的不用校验
        if !route.is_auth {
            log::debug!("路由 {} 未开启权限验证，无需鉴权", route.name);
            return;
        }
        // FIXME 修改匹配方式
        if route.auth_white_list.contains(&ctx.request.get_path()) {
            log::info!(
                "匹配到白名单，跳过鉴权，{} => {}",
                route.path,
                ctx.request.path
            );
            return;
        }

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

        if ApiKey::decrypt(ENCRYPT_KEY, api_key).is_err() {
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
