use crate::args::Args;
use crate::server::common::pool::HTTP_CLIENT;
use crate::server::db::Pool;
use crate::server::db::models::statistics_request_status_code::StatisticsRequestStatusCode;
use crate::server::db::models::system_config::{ConfigKey, SystemConfig};
use alert::Alert;
use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use logging::log;
use protocol::gateway::request_log::RequestLog;
use protocol::logg::LogSearchRes;
use serde_json::json;
use std::sync::Arc;

pub(crate) async fn request_status_count(args: Arc<Args>) {
    if let Err(e) = request_status_count_(args).await {
        log::error!("{}", e);
        Alert::error("定时任务【请求状态统计】执行异常", &e.to_string());
    }
}

pub(crate) async fn clean() {
    if let Err(e) = clean_().await {
        log::error!("{}", e);
        Alert::error("定时任务【请求状态统计数据清理】执行异常", &e.to_string());
    }
}

async fn request_status_count_(args: Arc<Args>) -> anyhow::Result<()> {
    log::debug!("[request_status_count] 状态码请求数统计开始执行");

    // 获取当前时间的前几分钟，复用
    let sub_minutes = |sub| {
        chrono::Local::now()
            .checked_sub_signed(chrono::Duration::minutes(sub))
            .unwrap()
            .with_second(0)
            .unwrap()
            .timestamp()
    };

    // 上次更新时间
    // 除第一次执行外，与当前时间差2分钟
    let last_timestamp = SystemConfig::get::<i64>(ConfigKey::RequestStatusCountLastUpdate).await?;

    // 如果是首次执行
    if last_timestamp == 0 {
        // 初始时间为当前时间减去1分钟，实现延迟统计
        // 因为请求日志写入有延时，如果立即统计上一分钟的，可能日志写入还未完成，导致遗漏
        let initial_time = sub_minutes(1);

        // 此时更新时间为：当前时间-1分钟，本次不会执行统计
        // 下次执行时，时间差将为2分钟，实际统计时间区间为：[前2分钟的0秒, 前2分钟的59秒]
        SystemConfig::upsert(ConfigKey::RequestStatusCountLastUpdate, &initial_time).await?;
        log::info!("[request_status_count] 首次执行，设置初始时间");
        return Ok(());
    }

    log::debug!(
        "[request_status_count] 上次统计区间: [{}, {}]",
        DateTime::from_timestamp_secs(last_timestamp)
            .unwrap()
            .with_timezone(&chrono_tz::Asia::Shanghai)
            .format("%Y-%m-%d %H:%M:%S"),
        DateTime::from_timestamp_secs(last_timestamp)
            .unwrap()
            .with_timezone(&chrono_tz::Asia::Shanghai)
            .with_second(59)
            .unwrap()
            .format("%Y-%m-%d %H:%M:%S")
    );

    // 注意这个now，不是当前时间，而是取当前时间减去1分钟，用来统计一分钟前的
    // 例如当前时间为：2025-12-01 11:26:32
    // 减去一分钟为：2025-12-01 11:25:32 = now
    // 此时last_time为：2025-12-01 11:24:32
    // 统计区间为[last_time, now]，即[2025-12-01 11:24:00, 2025-12-01 11:24:59]
    let now = sub_minutes(1);

    // 如果上次统计时间已经接近或超过当前延迟时间，则无需统计
    if last_timestamp >= now {
        log::info!("[request_status_count] 无需统计，上次统计时间已是最新的");
        return Ok(());
    }

    // 时间戳转时间
    let last_datetime: DateTime<Utc> = Utc.timestamp_opt(last_timestamp, 0).unwrap();
    let now_datetime: DateTime<Utc> = Utc.timestamp_opt(now, 0).unwrap();

    // 上次更新时间，调整到分钟的开始
    let mut last_datetime = last_datetime.with_second(0).unwrap();

    // 日志服务接口，目前支持quickwit和logg
    let api = format!(
        "http://{}/api/v1/{}/search",
        args.log_server, "request-logs"
    );

    let tx = Pool::get()?;

    while last_datetime < now_datetime {
        // 分钟整点时间戳
        let start_timestamp = last_datetime.timestamp();
        // 分钟结束时间戳
        let end_timestamp = last_datetime.with_second(59).unwrap().timestamp();

        // end_timestamp + 1 是为了兼容日志服务的查询，日志服务是左闭右开区间，不包含结束时间。
        let counts = search(&api, start_timestamp, end_timestamp + 1).await?;

        log::debug!(
            "[request_status_count] 区间 [{}, {}]，统计结果: {:?}",
            DateTime::from_timestamp_secs(start_timestamp)
                .unwrap()
                .with_timezone(&chrono_tz::Asia::Shanghai)
                .format("%Y-%m-%d %H:%M:%S"),
            DateTime::from_timestamp_secs(end_timestamp)
                .unwrap()
                .with_timezone(&chrono_tz::Asia::Shanghai)
                .format("%Y-%m-%d %H:%M:%S"),
            counts
        );

        let list = counts
            .iter()
            .map(|(status_code, count)| StatisticsRequestStatusCode {
                status_code: Some(*status_code),
                count: Some(*count),
                state_time: Some(start_timestamp),
            })
            .collect::<Vec<_>>();

        if !list.is_empty() {
            StatisticsRequestStatusCode::insert_batch(tx, &list, 1000).await?;
        }

        // 移动到下一分钟
        last_datetime += chrono::Duration::minutes(1);
    }

    // 最后再更新时间
    SystemConfig::upsert(ConfigKey::RequestStatusCountLastUpdate, &now).await?;

    Ok(())
}

async fn search(
    api: &str,
    start_timestamp: i64,
    end_timestamp: i64,
) -> anyhow::Result<Vec<(u16, i64)>> {
    let result = HTTP_CLIENT
        .post(api)
        .json(&json!({
            "query": "*",
            "start_timestamp": start_timestamp,
            "end_timestamp": end_timestamp,
            "start_offset": 0,
            "max_hits": 1,
            "aggs": {
                "count":{
                    "terms": {
                        "field": "status_code",
                        "size": 1000
                    }
                }
            }
        }))
        .send()
        .await?
        .json::<LogSearchRes<RequestLog>>()
        .await?;

    let buckets = result
        .aggregations
        .and_then(|aggs| aggs.get("count").cloned())
        .and_then(|count| count.get("buckets").unwrap_or_default().as_array().cloned())
        .unwrap_or_default();

    let counts = buckets
        .iter()
        .map(|bucket| {
            // status_code理论上是u16类型，但tantivy和quickwit在聚合后返回值类型不一致，
            // tantivy返回的是整数，而quickwit返回的是浮点数。
            // 所以这里统一先转f64再转u16
            let status_code = bucket.get("key").unwrap().as_f64().unwrap() as u16;
            let doc_count = bucket.get("doc_count").unwrap().as_i64().unwrap();
            (status_code, doc_count)
        })
        .collect::<Vec<(u16, i64)>>();

    Ok(counts)
}

async fn clean_() -> anyhow::Result<()> {
    log::info!("[request_status_count] 清理数据开始执行");

    // 一年前
    let one_year_ago = chrono::Local::now()
        .with_year(chrono::Local::now().year() - 1)
        .and_then(|dt| dt.with_second(0))
        .unwrap()
        .timestamp();

    let tx = Pool::get()?;

    let result = tx
        .exec(
            "DELETE FROM statistics_request_status_code WHERE state_time < ?",
            vec![one_year_ago.into()],
        )
        .await?;

    log::info!(
        "[request_status_count] 清理数据完成，删除了{}条数据",
        result.rows_affected
    );

    Ok(())
}
