use protocol::common::req::PageReq;
use protocol::impl_pagination;
use rbatis::rbdc::DateTime;
use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogListReq {
    page: PageReq,
    /// 模糊搜索：日志内容
    pub filter_text: Option<String>,
    /// 日志级别
    pub level: Option<String>,
    /// 起始时间
    pub start_time: Option<DateTime>,
    /// 结束时间
    pub end_time: Option<DateTime>,
}
impl_pagination!(LogListReq);
