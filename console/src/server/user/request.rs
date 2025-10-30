use protocol::common::req::PageReq;
use protocol::impl_pagination;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginReq {
    pub identity: String,
    pub secret: String,
    /// 登录方式：1密码登录 2邮箱验证码登陆
    pub login_type: i8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePasswordReq {
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendVerifyCodeReq {
    pub identity: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateEmailReq {
    pub email: String,
    pub secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserListReq {
    pub page: PageReq,
    pub filter_text: Option<String>,
}
impl_pagination!(UserListReq);

#[derive(Debug, Serialize, Deserialize)]
pub struct UserAddReq {
    pub username: String,
    pub password: String,
    pub nickname: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserUpdateReq {
    pub id: i64,
    pub nickname: Option<String>,
}
