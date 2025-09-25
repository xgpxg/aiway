use rocket::serde::{Deserialize, Serialize};

/// 缓存key
#[derive(strum_macros::Display)]
pub enum CacheKey {
    /// 用户Token，用于控制台登录
    /// 0: 用户Token
    #[strum(to_string = "aiway:user:token:{0}")]
    UserToken(String),

}

