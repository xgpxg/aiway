use crate::args::Args;
use crate::server::common::pool::HTTP_CLIENT;
use crate::server::db::Pool;
use crate::server::db::models::statistics_request_province::StatisticsRequestProvince;
use crate::server::db::models::system_config::{ConfigKey, SystemConfig};
use alert::Alert;
use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use logging::log;
use protocol::gateway::request_log::RequestLog;
use protocol::logg::LogSearchRes;
use rbs::value;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

pub(crate) async fn ip_region_count(args: Arc<Args>) {
    if let Err(e) = ip_region_count_(args).await {
        log::error!("{}", e);
        Alert::error("定时任务【区域调用统计】执行异常", &e.to_string());
    }
}

pub(crate) async fn clean() {
    if let Err(e) = clean_().await {
        log::error!("{}", e);
        Alert::error("定时任务【区域调用统计数据清理】执行异常", &e.to_string());
    }
}

/// 34个固定区域
#[rustfmt::skip]
const PROVINCES: [&str; 34] = ["北京", "广东省", "台湾省", "浙江省", "香港", "上海", "江苏省", "山东省", "辽宁省", "河北省", "河南省", "四川省", "江西省", "湖北省", "湖南省", "福建省", "重庆", "安徽省", "山西省", "吉林省", "陕西省", "天津", "广西", "黑龙江省", "新疆", "云南省", "内蒙古", "贵州省", "甘肃省", "海南省", "宁夏", "青海省", "西藏", "澳门"];

/// 获取IP对应的区域的调用次数
///
/// - 按小时级别统计，保留近1年的数据。
/// - 由定时任务清理1年以前的数据，每天执行一次。
async fn ip_region_count_(args: Arc<Args>) -> anyhow::Result<()> {
    log::info!("[ip_region_count] 区域请求统计开始执行");
    // 上次更新时间
    let last_time = SystemConfig::get::<i64>(ConfigKey::IpRegionLastUpdate).await?;
    if last_time == 0 {
        // 首次执行，设置初始时间
        SystemConfig::upsert(
            ConfigKey::IpRegionLastUpdate,
            &chrono::Local::now().timestamp(),
        )
        .await?;
        log::info!("[ip_region_count] 首次执行，设置初始时间");
        return Ok(());
    }

    log::info!("[ip_region_count] 上次更新时间: {}", last_time);

    // 当前时间
    let now = chrono::Local::now().timestamp();

    // 时间戳转时间
    let last_datetime: DateTime<Utc> = Utc.timestamp_opt(last_time, 0).unwrap();
    let now_datetime: DateTime<Utc> = Utc.timestamp_opt(now, 0).unwrap();

    // 上次更新时间，调整到小时的开始
    // 例如last_datetime为 2025-11-25 13:56:16，则调整为2025-11-25 13:00:00
    let mut last_datetime = last_datetime
        .with_minute(0)
        .and_then(|dt| dt.with_second(0))
        .unwrap();

    // 日志服务接口，目前支持quickwit和logg
    let api = format!(
        "http://{}/api/v1/{}/search",
        args.log_server, "request-logs"
    );

    let tx = Pool::get()?;

    while last_datetime < now_datetime {
        let start_timestamp = last_datetime.timestamp();
        let end_datetime = last_datetime + chrono::Duration::hours(1);
        let end_timestamp = end_datetime.timestamp();

        let counts = search(&api, start_timestamp, end_timestamp).await?;

        log::info!(
            "[ip_region_count] 区间 [{}, {}) 统计结果: {:?}",
            start_timestamp,
            end_timestamp,
            counts
        );

        let list = counts
            .iter()
            .map(|(province, count)| StatisticsRequestProvince {
                province: Some(province.to_string().replace("省", "")),
                count: Some(*count),
                start_time: Some(start_timestamp),
                end_time: Some(end_timestamp),
            })
            .collect::<Vec<_>>();
        StatisticsRequestProvince::delete_by_map(
            tx,
            value! {
                "start_time" : start_timestamp,
                "end_time" : end_timestamp,
            },
        )
        .await?;
        StatisticsRequestProvince::insert_batch(tx, &list, 50).await?;

        // 移动到下一小时
        last_datetime += chrono::Duration::hours(1);
    }

    // 最后再更新时间
    SystemConfig::upsert(ConfigKey::IpRegionLastUpdate, &now).await?;

    Ok(())
}

async fn search(
    api: &str,
    start_timestamp: i64,
    end_timestamp: i64,
) -> anyhow::Result<Vec<(String, i64)>> {
    let result = HTTP_CLIENT
        .post(api)
        .json(&json!({
            "query": "client_country:中国",
            "start_timestamp": start_timestamp,
            "end_timestamp": end_timestamp,
            "start_offset": 0,
            "max_hits": 1,
            "aggs": {
                "count": {
                    "terms": { "field": "client_province","size":100 }
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

    let count_map = buckets
        .iter()
        .map(|bucket| {
            let province = bucket.get("key").unwrap().as_str().unwrap();
            let doc_count = bucket.get("doc_count").unwrap().as_i64().unwrap();
            (province.to_string(), doc_count)
        })
        .collect::<HashMap<String, i64>>();

    let counts = PROVINCES
        .iter()
        .map(|province| {
            if let Some(count) = count_map.get(*province) {
                (province.replace("省", ""), *count)
            } else {
                (province.replace("省", ""), 0)
            }
        })
        .collect::<Vec<_>>();

    Ok(counts)
}

async fn clean_() -> anyhow::Result<()> {
    log::info!("[ip_region_count] 清理数据开始执行");

    // 一年前
    let one_year_ago = chrono::Local::now()
        .with_year(chrono::Local::now().year() - 1)
        .and_then(|dt| dt.with_minute(0))
        .and_then(|dt| dt.with_second(0))
        .unwrap()
        .timestamp();
    let result = StatisticsRequestProvince::delete_by_map(
        Pool::get()?,
        value! {
            "start_time" : one_year_ago,
        },
    )
    .await?;

    log::info!(
        "[ip_region_count] 清理数据完成，删除了{}条数据",
        result.rows_affected
    );

    Ok(())
}
