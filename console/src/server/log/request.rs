use chrono::NaiveDateTime;
use busi::req::PageReq;
use busi::impl_pagination;
use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogListReq {
    page: PageReq,
    /// 模糊搜索：日志内容
    pub filter_text: Option<String>,
    /// 日志级别
    pub level: Option<String>,
    /// 起始时间
    pub start_time: Option<NaiveDateTime>,
    /// 结束时间
    pub end_time: Option<NaiveDateTime>,
}
impl_pagination!(LogListReq);
