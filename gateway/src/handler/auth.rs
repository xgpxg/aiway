//! # 鉴权 - API Key 验证
//!
use crate::components::Firewalld;
use crate::handler::{HttpError, HttpResult};
use aiway_protocol::common::header::Headers;
use aiway_protocol::context::{HttpContext, RequestExt};
use aiway_protocol::gateway::ApiKey;
use cache::caches::CacheKey;
use pingora::prelude::*;

const BEARER_PREFIX: &str = "Bearer ";

pub async fn auth_handle(session: &mut Session, ctx: &HttpContext) -> HttpResult<()> {
    // 获取上下文
    // SAFE: 此时路由一定存在
    let route = ctx.get_route().unwrap();

    // 未开启权限验证的不用校验
    if !route.is_auth {
        log::debug!("路由 {} 未开启权限验证，无需鉴权", route.name);
        return Ok(());
    }
    // FIXME 修改匹配方式
    if route.auth_white_list.contains(&session.req_header().get_path()) {
        log::debug!(
            "匹配到白名单，跳过鉴权，{} => {}",
            route.path,
            session.req_header().get_path()
        );
        return Ok(());
    }

    let bearer_token = session.req_header().headers.get(Headers::AUTHORIZATION);

    let api_key = match bearer_token {
        Some(api_key) => match api_key.to_str()?.strip_prefix(BEARER_PREFIX) {
            Some(api_key) => api_key,
            None => {
                return Err(HttpError::new(401, "Unauthorized"));
            }
        },
        None => {
            return Err(HttpError::new(401, "Unauthorized"));
        }
    };

    let decrypt_key = &Firewalld::get_api_secret_encrypt_key().await;
    if ApiKey::decrypt(decrypt_key, api_key).is_err() {
        return Err(HttpError::new(401, "Unauthorized"));
    }

    let exists = cache::exists(&CacheKey::ApiKey(api_key.to_string()).to_string())
        .await
        .unwrap_or(false);
    if !exists {
        return Err(HttpError::new(401, "Unauthorized"));
    }
    Ok(())
}
