use derive_builder::Builder;
use rbatis::crud;
use rbatis::executor::Executor;
use rbatis::htmlsql;
use rocket::serde::{Deserialize, Serialize};

/// 模型调用统计（分钟级明细）
#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
#[builder(default)]
pub struct StatisticsModelCall {
    /// 模型名称
    pub model_name: Option<String>,
    /// 提供商名称
    pub provider_name: Option<String>,
    /// 分钟级起始时间戳
    pub state_time: Option<i64>,
    /// 该分钟调用次数
    pub call_count: Option<i64>,
    /// 该分钟 prompt tokens
    pub prompt_tokens: Option<i64>,
    /// 该分钟 completion tokens
    pub completion_tokens: Option<i64>,
    /// 该分钟 total tokens
    pub tokens: Option<i64>,
    /// 该分钟平均耗时（毫秒）
    pub avg_elapsed: Option<i64>,
    /// 该分钟平均首 Token 耗时（毫秒）
    pub avg_ttft: Option<i64>,
}

crud!(StatisticsModelCall {});

/// 模型调用累计汇总（每个 model+provider 仅一行）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(dead_code)]
pub struct StatisticsModelCallTotal {
    pub model_name: String,
    pub provider_name: String,
    pub total_call_count: i64,
    pub total_prompt_tokens: i64,
    pub total_completion_tokens: i64,
    pub total_tokens: i64,
}

crud!(StatisticsModelCallTotal {}, "statistics_model_call_total");

// ---------- 查询参数 ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankParam {
    pub field: String,
    pub start_time: i64,
    pub end_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start_time: i64,
    pub end_time: i64,
}

// ---------- 查询返回 ----------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TotalSummary {
    pub total_tokens: i64,
    pub total_call_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TodaySummary {
    pub tokens: i64,
    pub call_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AvgElapsedRes {
    pub avg_elapsed: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActiveModelsRes {
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrendRow {
    pub state_time: i64,
    pub tokens: i64,
    pub call_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RankRow {
    pub name: String,
    pub value: i64,
}

// ---------- Mapper ----------

htmlsql!(overview_total(rb: &dyn Executor) -> TotalSummary => "src/server/db/mapper/statistics_model_call.html");
htmlsql!(overview_today(rb: &dyn Executor, today_start: i64) -> TodaySummary => "src/server/db/mapper/statistics_model_call.html");
htmlsql!(overview_avg_elapsed(rb: &dyn Executor, five_min_ago: i64) -> AvgElapsedRes => "src/server/db/mapper/statistics_model_call.html");
htmlsql!(overview_active_models(rb: &dyn Executor, today_start: i64) -> ActiveModelsRes => "src/server/db/mapper/statistics_model_call.html");
htmlsql!(select_trend(rb: &dyn Executor, param: &TimeRange) -> Vec<TrendRow> => "src/server/db/mapper/statistics_model_call.html");
htmlsql!(select_model_rank(rb: &dyn Executor, param: &RankParam) -> Vec<RankRow> => "src/server/db/mapper/statistics_model_call.html");