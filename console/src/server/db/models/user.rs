use derive_builder::Builder;
use rbatis::crud;
use rbatis::rbdc::DateTime;
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
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新时间
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
