use crate::args::Args;
use crate::server::common::pool::HTTP_CLIENT;
use crate::server::log::request::LogListReq;
use chrono::TimeZone;
use aiway_protocol::common::req::Pagination;
use aiway_protocol::common::res::PageRes;
use aiway_protocol::gateway::request_log::RequestLog;
use aiway_protocol::logg::{LogEntry, LogSearchReq, LogSearchRes};
use rocket::State;

const AIWAY_LOG_INDEX: &str = "aiway-logs";
const REQUEST_LOG_INDEX: &str = "request-logs";

pub async fn list(req: LogListReq, args: &State<Args>) -> anyhow::Result<PageRes<LogEntry>> {
    let log_server = &args.log_server;

    let url = format!("http://{}/api/v1/{}/search", log_server, AIWAY_LOG_INDEX);

    let mut query = Vec::new();
    if let Some(filter_text) = &req.filter_text
        && !filter_text.is_empty()
    {
        query.push(format!("message:{}", filter_text));
    }
    if let Some(level) = &req.level {
        query.push(format!("level:{}", level));
    }

    let start_offset = ((req.page_num() - 1) * req.page_size()) as usize;
    let max_hits = req.page_size() as usize;

    let param = LogSearchReq {
        query: Some(query.join(" AND ")),
        start_timestamp: req
            .start_time
            .map(|t| chrono::Utc.from_utc_datetime(&t).timestamp()),
        end_timestamp: req
            .end_time
            .map(|t| chrono::Utc.from_utc_datetime(&t).timestamp()),
        start_offset,
        max_hits,
        aggs: None,
        sort_by: Some("time".into()),
    };

    let res = HTTP_CLIENT
        .post(url)
        .json(&param)
        .send()
        .await?
        .json::<LogSearchRes<LogEntry>>()
        .await?;

    Ok(PageRes {
        page_num: req.page_num(),
        page_size: req.page_size(),
        total: res.num_hits as u64,
        list: res.hits,
        ext: None,
    })
}

pub(crate) async fn request_log_list(
    req: LogListReq,
    args: &State<Args>,
) -> anyhow::Result<PageRes<RequestLog>> {
    let log_server = &args.log_server;

    let url = format!("http://{}/api/v1/{}/search", log_server, REQUEST_LOG_INDEX);

    let mut query = Vec::new();
    if let Some(filter_text) = &req.filter_text
        && !filter_text.is_empty()
    {
        query.push(filter_text.clone());
    }

    let start_offset = ((req.page_num() - 1) * req.page_size()) as usize;
    let max_hits = req.page_size() as usize;

    let param = LogSearchReq {
        query: Some(query.join(" AND ")),
        start_timestamp: req
            .start_time
            .map(|t| chrono::Utc.from_utc_datetime(&t).timestamp()),
        end_timestamp: req
            .end_time
            .map(|t| chrono::Utc.from_utc_datetime(&t).timestamp()),
        start_offset,
        max_hits,
        aggs: None,
        sort_by: Some("request_time".into()),
    };

    let res = HTTP_CLIENT
        .post(url)
        .json(&param)
        .send()
        .await?
        .json::<LogSearchRes<RequestLog>>()
        .await?;

    Ok(PageRes {
        page_num: req.page_num(),
        page_size: req.page_size(),
        total: res.num_hits as u64,
        list: res.hits,
        ext: None,
    })
}
