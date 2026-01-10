use derive_builder::Builder;
use rbatis::crud;
use rbatis::rbdc::DateTime;
use serde::{Deserialize, Serialize};
use aiway_protocol::gateway::ConfiguredPlugin;

/// 模型提供商配置
#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
#[builder(default)]
pub struct ModelProvider {
    pub id: Option<i64>,
    /// 模型ID
    pub model_id: Option<i64>,
    /// 模型提供商名称
    pub name: Option<String>,
    /// 接口地址
    pub api_url: Option<String>,
    /// 密钥
    pub api_key: Option<String>,
    /// 状态：Disable | Ok
    pub status: Option<ModelProviderStatus>,
    /// 权重
    pub weight: Option<u32>,
    /// 目标模型名称
    ///
    /// 场景：对于同一个模型在不同提供商处的名字可能不一样，
    /// 通过该模型名称映射提供商处的真实模型名称
    pub target_model_name: Option<String>,
    /// 请求转换插件
    pub request_converter: Option<ConfiguredPlugin>,
    /// 响应转换插件
    pub response_converter: Option<ConfiguredPlugin>,
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
pub enum ModelProviderStatus {
    /// 停用
    #[default]
    Disable,
    /// 启用
    Ok,
}
crud!(ModelProvider {});
