use protocol::common::req::PageReq;
use protocol::impl_pagination;
use serde::{Deserialize, Serialize};
use validator::Validate;

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
#[allow(unused)]
pub struct SendVerifyCodeReq {
    pub identity: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(unused)]
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

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UserAddReq {
    #[validate(length(min = 1, max = 50, message = "长度必须大于1且小于50"))]
    pub username: String,
    pub password: String,
    #[validate(length(min = 1, max = 50, message = "长度必须大于1且小于50"))]
    pub nickname: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserUpdateReq {
    pub id: i64,
    pub nickname: Option<String>,
}
