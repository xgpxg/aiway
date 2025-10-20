use crate::args::Args;
use crate::server::common::pool::HTTP_CLIENT;
use crate::server::log::request::LogListReq;
use protocol::common::req::Pagination;
use protocol::common::res::PageRes;
use protocol::logg::{LogEntry, LogSearchReq, LogSearchRes};
use rocket::State;

pub async fn list(req: LogListReq, args: &State<Args>) -> anyhow::Result<PageRes<LogEntry>> {
    let log_server = &args.log_server;

    let url = format!("http://{}/api/v1/aiway-logs/search", log_server);
    println!("url: {}", url);

    let mut query = Vec::new();
    if let Some(filter_text) = &req.filter_text {
        if !filter_text.is_empty() {
            query.push(format!("message:{}", filter_text));
        }
    }
    if let Some(level) = &req.level {
        query.push(format!("level:{}", level));
    }

    let start_offset = ((req.page_num() - 1) * req.page_size()) as usize;
    let max_hits = req.page_size() as usize;

    let param = LogSearchReq {
        query: Some(query.join(" AND ")),
        start_timestamp: req.start_time.clone().map(|t| t.unix_timestamp()),
        end_timestamp: req.end_time.clone().map(|t| t.unix_timestamp()),
        start_offset,
        max_hits,
    };

    let res = HTTP_CLIENT
        .post(url)
        .json(&param)
        .send()
        .await?
        .json::<LogSearchRes>()
        .await?;

    Ok(PageRes {
        page_num: req.page_num(),
        page_size: req.page_size(),
        total: res.num_hits as u64,
        list: res.hits,
    })
}
