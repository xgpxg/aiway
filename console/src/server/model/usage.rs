use crate::server::db::Pool;
use crate::server::db::models::statistics_model_call::{self, RankParam, TimeRange};
use crate::server::model::request::{RankReq, RankType, TrendReq};
use crate::server::model::response::{ModelRankItem, TrendItem, UsageOverview};
use chrono::Timelike;

pub(crate) async fn overview() -> anyhow::Result<UsageOverview> {
    let tx = Pool::get()?;

    let today_start = chrono::Local::now()
        .with_hour(0)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap()
        .timestamp();
    let five_min_ago = (chrono::Local::now() - chrono::Duration::minutes(5)).timestamp();

    let total = statistics_model_call::overview_total(tx)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    let today = statistics_model_call::overview_today(tx, today_start)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    let avg_elapsed = statistics_model_call::overview_avg_elapsed(tx, five_min_ago)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    let active_models = statistics_model_call::overview_active_models(tx, today_start)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    Ok(UsageOverview {
        total_tokens: total.total_tokens,
        total_call_count: total.total_call_count,
        today_tokens: today.tokens,
        today_call_count: today.call_count,
        avg_elapsed: avg_elapsed.avg_elapsed,
        active_models: active_models.count,
    })
}

pub(crate) async fn trend(req: TrendReq) -> anyhow::Result<Vec<TrendItem>> {
    let tx = Pool::get()?;
    let rows = statistics_model_call::select_trend(
        tx,
        &TimeRange {
            start_time: req.start_timestamp,
            end_time: req.end_timestamp,
        },
    )
    .await
    .map_err(|e| anyhow::anyhow!(e))?;
    Ok(rows
        .into_iter()
        .map(|r| TrendItem {
            time: r.state_time,
            tokens: r.tokens,
            call_count: r.call_count,
        })
        .collect())
}

pub(crate) async fn model_rank(req: RankReq) -> anyhow::Result<Vec<ModelRankItem>> {
    let req_end_ts = req
        .end_timestamp
        .unwrap_or_else(|| chrono::Local::now().timestamp());
    let today_start = chrono::Local::now()
        .with_hour(0)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap()
        .timestamp();
    let end_ts = req_end_ts.max(today_start);
    let start_ts = req.start_timestamp.unwrap_or(today_start);
    let duration = end_ts - start_ts;

    let field = match req.r#type {
        RankType::Calls => "call_count",
        RankType::Tokens => "tokens",
    };

    let tx = Pool::get()?;
    let curr = statistics_model_call::select_model_rank(
        tx,
        &RankParam {
            field: field.to_string(),
            start_time: start_ts,
            end_time: end_ts,
        },
    )
    .await
    .map_err(|e| anyhow::anyhow!(e))?;

    let prev = statistics_model_call::select_model_rank(
        tx,
        &RankParam {
            field: field.to_string(),
            start_time: start_ts - duration,
            end_time: start_ts,
        },
    )
    .await
    .map_err(|e| anyhow::anyhow!(e))?;

    let prev_map: std::collections::HashMap<String, i64> =
        prev.into_iter().map(|r| (r.name, r.value)).collect();

    Ok(curr
        .into_iter()
        .map(|r| {
            let change = prev_map.get(&r.name).and_then(|prev| {
                if *prev > 0 {
                    Some(((r.value - prev) * 100) / prev)
                } else {
                    None
                }
            });
            ModelRankItem {
                name: r.name,
                value: r.value,
                change,
            }
        })
        .collect())
}
