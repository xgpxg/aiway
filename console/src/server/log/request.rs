use protocol::common::req::PageReq;
use protocol::impl_pagination;
use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogListReq {
    page: PageReq,
    /// 模糊搜索：日志内容
    pub filter_text: Option<String>,
    /// 日志级别
    pub level: Option<String>,
    /// 时间段
    pub time_range: Option<(Option<String>, Option<String>)>,
}
impl_pagination!(LogListReq);
