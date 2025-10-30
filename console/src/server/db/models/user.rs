use crate::server::user::{UserListReq, UserListRes};
use derive_builder::Builder;
use rbatis::rbdc::DateTime;
use rbatis::{crud, htmlsql_select_page};
use rocket::serde::{Deserialize, Serialize};

/// 用户
#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
#[builder(default)]
pub struct User {
    /// 用户ID
    pub id: Option<i64>,
    /// 昵称
    pub nickname: Option<String>,
    /// 头像
    pub avatar: Option<String>,
    /// 状态：0未生效 1正常 9冻结
    pub status: Option<i8>,
    /// 最后登录时间
    #[serde(serialize_with = "crate::server::common::serialize_datetime")]
    pub last_login_time: Option<DateTime>,
    /// 创建时间
    #[serde(serialize_with = "crate::server::common::serialize_datetime")]
    pub create_time: Option<DateTime>,
    /// 更新时间
    #[serde(serialize_with = "crate::server::common::serialize_datetime")]
    pub update_time: Option<DateTime>,
    /// 备注
    pub remark: Option<String>,
    /// 是否删除
    pub is_delete: Option<i8>,
}

#[allow(unused)]
pub enum UserStatus {
    /// 未生效
    NotEffective = 0,
    /// 正常
    Ok = 1,
    /// 冻结
    Frozen = 9,
}

crud!(User {});
htmlsql_select_page!(list_page(param: &UserListReq) -> UserListRes => "src/server/db/mapper/user.html");
