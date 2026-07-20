use crate::args::Args;
use crate::server::common::pool::HTTP_CLIENT;
use crate::server::db::Pool;
use crate::server::db::models::statistics_model_call::StatisticsModelCall;
use crate::server::db::models::system_config::{ConfigKey, SystemConfig};
use alert::Alert;
use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use logging::log;
use aiway_protocol::gateway::ModelCallLog;
use aiway_protocol::logg::LogSearchRes;
use rbs::value;
use serde_json::json;
use std::sync::Arc;

pub(crate) async fn model_call_count(args: Arc<Args>) {
    if let Err(e) = model_call_count_(args).await {
        log::error!("{}", e);
        Alert::error("定时任务【模型调用统计】执行异常", &e.to_string());
    }
}

pub(crate) async fn clean() {
    if let Err(e) = clean_().await {
        log::error!("{}", e);
        Alert::error("定时任务【模型调用统计数据清理】执行异常", &e.to_string());
    }
}

async fn model_call_count_(args: Arc<Args>) -> anyhow::Result<()> {
    log::debug!("[model_call_count] 模型调用统计开始执行");

    let sub_minutes = |sub: i64| {
        chrono::Local::now()
            .checked_sub_signed(chrono::Duration::minutes(sub))
            .unwrap()
            .with_second(0)
            .unwrap()
            .timestamp()
    };

    let last_timestamp = SystemConfig::get::<i64>(ConfigKey::ModelCallCountLastUpdate).await?;

    if last_timestamp == 0 {
        let initial_time = sub_minutes(1);
        SystemConfig::upsert(ConfigKey::ModelCallCountLastUpdate, &initial_time).await?;
        log::info!("[model_call_count] 首次执行，设置初始时间");
        return Ok(());
    }

    let now = sub_minutes(1);

    if last_timestamp >= now {
        log::info!("[model_call_count] 无需统计，上次统计时间已是最新的");
        return Ok(());
    }

    let last_datetime: DateTime<Utc> = Utc.timestamp_opt(last_timestamp, 0).unwrap();
    let now_datetime: DateTime<Utc> = Utc.timestamp_opt(now, 0).unwrap();
    let mut last_datetime = last_datetime.with_second(0).unwrap();

    let api = format!(
        "http://{}/api/v1/{}/search",
        args.log_server, "model-call-logs"
    );

    let tx = Pool::get()?;

    while last_datetime < now_datetime {
        let start_timestamp = last_datetime.timestamp();
        let end_timestamp = last_datetime.with_second(59).unwrap().timestamp();

        let stats = search(&api, start_timestamp, end_timestamp + 1).await?;

        if !stats.is_empty() {
            // 覆盖写入：先删除该分钟已有数据
            tx.exec(
                "DELETE FROM statistics_model_call WHERE state_time = ?",
                vec![start_timestamp.into()],
            )
            .await?;

            // 写入分钟级明细
            let list: Vec<StatisticsModelCall> = stats
                .iter()
                .map(|s| StatisticsModelCall {
                    model_name: Some(s.model_name.clone()),
                    provider_name: Some(s.provider_name.clone()),
                    state_time: Some(start_timestamp),
                    call_count: Some(s.call_count),
                    prompt_tokens: Some(s.prompt_tokens),
                    completion_tokens: Some(s.completion_tokens),
                    tokens: Some(s.tokens),
                    avg_elapsed: Some(s.avg_elapsed),
                    avg_ttft: Some(s.avg_ttft),
                })
                .collect();

            StatisticsModelCall::insert_batch(tx, &list, 1000).await?;

            // UPSERT 累计表
            for s in &stats {
                tx.exec(
                    "INSERT INTO statistics_model_call_total \
                     (model_name, provider_name, total_call_count, total_prompt_tokens, total_completion_tokens, total_tokens) \
                     VALUES (?, ?, ?, ?, ?, ?) \
                     ON CONFLICT(model_name, provider_name) DO UPDATE SET \
                     total_call_count = total_call_count + excluded.total_call_count, \
                     total_prompt_tokens = total_prompt_tokens + excluded.total_prompt_tokens, \
                     total_completion_tokens = total_completion_tokens + excluded.total_completion_tokens, \
                     total_tokens = total_tokens + excluded.total_tokens",
                    vec![
                        value!(s.model_name.clone()),
                        value!(s.provider_name.clone()),
                        value!(s.call_count),
                        value!(s.prompt_tokens),
                        value!(s.completion_tokens),
                        value!(s.tokens),
                    ],
                )
                .await?;
            }
        }

        last_datetime += chrono::Duration::minutes(1);
    }

    SystemConfig::upsert(ConfigKey::ModelCallCountLastUpdate, &now).await?;
    Ok(())
}

/// 单个模型的分钟级聚合统计结果
#[derive(Debug, Default)]
struct ModelStats {
    model_name: String,
    provider_name: String,
    call_count: i64,
    prompt_tokens: i64,
    completion_tokens: i64,
    tokens: i64,
    avg_elapsed: i64,
    avg_ttft: i64,
}

async fn search(api: &str, start_timestamp: i64, end_timestamp: i64) -> anyhow::Result<Vec<ModelStats>> {
    // 使用嵌套 terms 聚合按 model_name 分组，再按 provider_name 分组
    let result = HTTP_CLIENT
        .post(api)
        .json(&json!({
            "query": "*",
            "start_timestamp": start_timestamp,
            "end_timestamp": end_timestamp,
            "start_offset": 0,
            "max_hits": 1,
            "aggs": {
                "by_model": {
                    "terms": { "field": "model_name", "size": 1000 },
                    "aggs": {
                        "by_provider": {
                            "terms": { "field": "provider_name", "size": 1000 },
                            "aggs": {
                                "sum_prompt_tokens": { "sum": { "field": "prompt_tokens" } },
                                "sum_completion_tokens": { "sum": { "field": "completion_tokens" } },
                                "sum_total_tokens": { "sum": { "field": "total_tokens" } },
                                "avg_elapsed": { "avg": { "field": "elapsed" } },
                                "avg_ttft": { "avg": { "field": "ttft_ms" } }
                            }
                        }
                    }
                }
            }
        }))
        .send()
        .await?
        .json::<LogSearchRes<ModelCallLog>>()
        .await?;

    let model_buckets = result
        .aggregations
        .and_then(|aggs| aggs.get("by_model").cloned())
        .and_then(|m| m.get("buckets").cloned())
        .and_then(|b| b.as_array().cloned())
        .unwrap_or_default();

    let stats = model_buckets
        .iter()
        .flat_map(|mb| {
            let model_name = mb.get("key")?.as_str()?.to_string();
            let provider_buckets = mb
                .get("by_provider")?
                .get("buckets")?
                .as_array()?
                .clone();
            let model_name_clone = model_name.clone();
            Some(
                provider_buckets
                    .into_iter()
                    .filter_map(move |pb| {
                        let provider_name = pb.get("key")?.as_str()?.to_string();
                        let call_count = pb.get("doc_count").and_then(|v| v.as_i64()).unwrap_or(0);
                        Some(ModelStats {
                            model_name: model_name_clone.clone(),
                            provider_name,
                            call_count,
                            prompt_tokens: extract_agg_value(&pb, "sum_prompt_tokens"),
                            completion_tokens: extract_agg_value(&pb, "sum_completion_tokens"),
                            tokens: extract_agg_value(&pb, "sum_total_tokens"),
                            avg_elapsed: extract_agg_value(&pb, "avg_elapsed"),
                            avg_ttft: extract_agg_value(&pb, "avg_ttft"),
                        })
                    }),
            )
        })
        .flatten()
        .collect();

    Ok(stats)
}

/// 从聚合结果中提取数值（兼容 f64 和 i64）
fn extract_agg_value(bucket: &serde_json::Value, agg_name: &str) -> i64 {
    bucket
        .get(agg_name)
        .and_then(|v| v.get("value"))
        .and_then(|v| v.as_f64())
        .map(|v| v as i64)
        .unwrap_or(0)
}

async fn clean_() -> anyhow::Result<()> {
    log::debug!("[model_call_count] 清理数据开始执行");

    let one_year_ago = chrono::Local::now()
        .with_year(chrono::Local::now().year() - 1)
        .and_then(|dt| dt.with_second(0))
        .unwrap()
        .timestamp();

    let tx = Pool::get()?;
    let result = tx
        .exec(
            "DELETE FROM statistics_model_call WHERE state_time < ?",
            vec![one_year_ago.into()],
        )
        .await?;

    log::debug!(
        "[model_call_count] 清理数据完成，删除了{}条数据",
        result.rows_affected
    );

    Ok(())
}
