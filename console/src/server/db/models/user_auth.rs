use derive_builder::Builder;
use rbatis::crud;
use rbatis::rbdc::DateTime;
use rocket::serde::{Deserialize, Serialize};

/// 用户认证
#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
#[builder(default)]
pub struct UserAuth {
    pub id: Option<i64>,
    /// 身份类型：1手机号 2微信openid 3用户名/密码 4邮箱
    pub r#type: Option<i8>,
    /// 身份标识
    pub identity: Option<String>,
    /// 身份密钥
    pub secret: Option<String>,
    /// 创建人ID
    pub create_user_id: Option<i64>,
    /// 修改人ID
    pub update_user_id: Option<i64>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
    /// 备注
    pub remark: Option<String>,
    /// 用户ID
    pub user_id: Option<i64>,
    /// 是否删除
    pub is_delete: Option<i8>,
}
#[allow(unused)]
pub enum IdentityType {
    Username = 1,
    Email = 2,
}

crud!(UserAuth {});
