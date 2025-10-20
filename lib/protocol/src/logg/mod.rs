use rocket::serde::Serialize;
use serde::Deserialize;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LogEntry {
    pub time: String,
    pub service: String,
    pub level: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct LogSearchReq {
    /// 查询参数，例如：level:INFO
    /// 查询语法参考：https://quickwit.io/docs/reference/query-language
    pub query: Option<String>,
    /// 秒级时间戳，包含
    pub start_timestamp: Option<i64>,
    /// 秒级时间戳，不包含
    pub end_timestamp: Option<i64>,
    /// 查询起始位置
    #[serde(default = "LogSearchReq::default_start_offset")]
    pub start_offset: usize,
    /// 查询最大数量
    #[serde(default = "LogSearchReq::default_max_hits")]
    pub max_hits: usize,
}
impl LogSearchReq {
    fn default_start_offset() -> usize {
        0
    }
    fn default_max_hits() -> usize {
        10
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct LogSearchRes {
    pub num_hits: usize,
    pub hits: Vec<LogEntry>,
}
