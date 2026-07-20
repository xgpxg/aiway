use crate::server::db::models::model::ModelStatus;
use crate::server::db::models::model_provider::ModelProviderStatus;
use aiway_protocol::gateway::ConfiguredPlugin;
use aiway_protocol::model::{LbStrategy, TokenUsageConfig};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ModelLisReq {}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelAddReq {
    pub name: String,
}
#[derive(Debug, Clone, Deserialize)]
pub struct ModelUpdateReq {
    pub id: i64,
    pub name: Option<String>,
    pub status: Option<ModelStatus>,
    pub lb_strategy: Option<LbStrategy>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct ProviderAddReq {
    pub model_id: i64,
    pub name: String,
    pub api_url: String,
    pub api_key: Option<String>,
    pub weight: Option<u32>,
    pub target_model_name: Option<String>,
    pub plugins: Option<ConfiguredPlugin>,
    pub token_usage_config: Option<TokenUsageConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProviderUpdateReq {
    pub id: i64,
    pub name: Option<String>,
    pub api_url: Option<String>,
    pub api_key: Option<String>,
    pub status: Option<ModelProviderStatus>,
    pub weight: Option<u32>,
    pub target_model_name: Option<String>,
    pub plugins: Option<ConfiguredPlugin>,
    pub token_usage_config: Option<TokenUsageConfig>,
}

// -- 用量统计 --

#[derive(Debug, Clone, Deserialize)]
pub struct TrendReq {
    pub start_timestamp: i64,
    pub end_timestamp: i64,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RankType {
    Calls,
    Tokens,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RankReq {
    pub r#type: RankType,
    pub start_timestamp: Option<i64>,
    pub end_timestamp: Option<i64>,
}
