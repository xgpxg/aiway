use derive_builder::Builder;
use rbatis::crud;
use rbatis::rbdc::DateTime;
use rocket::serde::{Deserialize, Serialize};

/// 系统设置
#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
#[builder(default)]
pub struct Setting {
    pub id: Option<i64>,
    /// 配置Key
    pub config_key: Option<String>,
    /// 配置值
    #[serde(deserialize_with = "crate::server::common::deserialize_to_string")]
    pub config_value: Option<String>,
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
    /// 是否删除
    pub is_delete: Option<i8>,
}

crud!(Setting {});

