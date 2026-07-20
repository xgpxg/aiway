use crate::server::db::models::model::Model;
use crate::server::db::models::model_provider::ModelProvider;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ModelListRes {
    #[serde(flatten)]
    pub inner: Model,
    pub providers: Vec<ModelProvider>,
}

/// 用量统计
#[derive(Debug, Clone, Serialize, Default)]
pub struct UsageOverview {
    pub total_tokens: i64,
    pub total_call_count: i64,
    pub today_tokens: i64,
    pub today_call_count: i64,
    pub avg_elapsed: i64,
    pub active_models: i64,
}

/// 趋势项
#[derive(Debug, Clone, Serialize)]
pub struct TrendItem {
    pub time: i64,
    pub tokens: i64,
    pub call_count: i64,
}

/// 排行项
#[derive(Debug, Clone, Serialize)]
pub struct ModelRankItem {
    pub name: String,
    pub value: i64,
    pub change: Option<i64>,
}
