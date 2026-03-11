//! # 鉴权
//! ## 主要功能
//! 从请求中提取ApiKey并验证，验证不通过则返回403。
//!
//! 考虑是调用另外的服务验证，还是对API Key解密验证?
//!
use crate::components::Firewalld;
use aiway_protocol::gateway::ApiKey;
use cache::caches::CacheKey;
use context::{HCM, Headers, set_error, skip_if_error};
use rocket::fairing::Fairing;
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
            log::debug!(
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

        let decrypt_key = &Firewalld::get_api_secret_encrypt_key().await;
        if ApiKey::decrypt(decrypt_key, api_key).is_err() {
            set_error!(req, 401, "Unauthorized");
            return;
        }

        let exists = cache::exists(&CacheKey::ApiKey(api_key.to_string()).to_string())
            .await
            .unwrap_or(false);
        if !exists {
            set_error!(req, 401, "Unauthorized");
            return;
        }
    }
}
