use derive_builder::Builder;
use rbatis::crud;
use rbatis::rbdc::DateTime;
use rocket::serde::{Deserialize, Serialize};
use protocol::model::LbStrategy;

/// 模型配置
#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
#[builder(default)]
pub struct Model {
    pub id: Option<i64>,
    /// 模型名称，全局唯一
    pub name: Option<String>,
    /// 状态：Disable | Ok
    pub status: Option<ModelStatus>,
    /// 负载均衡策略：RoundRobin | Random | WeightedRandom
    pub lb_strategy: Option<LbStrategy>,
    /// 创建人ID
    pub create_user_id: Option<i64>,
    /// 修改人ID
    pub update_user_id: Option<i64>,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ModelStatus {
    /// 停用
    #[default]
    Disable,
    /// 启用
    Ok,
}


crud!(Model {});
