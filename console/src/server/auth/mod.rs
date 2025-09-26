//! Token验证

use logging::log;
use rocket::Request;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use serde::{Deserialize, Serialize};
use cache::caches::CacheKey;

/// 已登录的用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPrincipal {
    /// 用户ID
    pub id: i64,
    /// 昵称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,

    /// 用户Token，该值不会被序列化，仅在请求时从header中获取。
    /// 一方面保证安全性，另一方面减小缓存占用
    #[serde(skip_serializing)]
    pub token: Option<String>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserPrincipal {
    type Error = &'r str;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // 从header提取Token
        let token = req.headers().get_one("Authorization");
        let token = match token {
            Some(token) => match token.trim().split(' ').nth(1) {
                None => return Outcome::Error((Status::Unauthorized, "Need Login")),
                Some(token) => token,
            },
            None => return Outcome::Error((Status::Unauthorized, "Need Login")),
        };

        let mut user =
            match cache::get::<UserPrincipal>(&CacheKey::UserToken(token.to_string()).to_string())
                .await
            {
                Ok(value) => match value {
                    Some(value) => value,
                    None => return Outcome::Error((Status::Unauthorized, "Need Login")),
                },
                Err(e) => {
                    log::error!("get token error: {}", e);
                    return Outcome::Error((Status::Unauthorized, "Need Login"));
                }
            };

        user.token = Some(token.to_string());

        Outcome::Success(user)
    }
}
