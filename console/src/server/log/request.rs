use busi::impl_pagination;
use busi::req::PageReq;
use chrono::NaiveDateTime;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCallLogListReq {
    page: PageReq,
    /// 模型名称过滤
    pub model_name: Option<String>,
    /// 提供商名称过滤
    pub provider_name: Option<String>,
    /// 起始时间
    pub start_time: Option<NaiveDateTime>,
    /// 结束时间
    pub end_time: Option<NaiveDateTime>,
}
impl_pagination!(ModelCallLogListReq);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteLogReq {
    pub start_time: Option<NaiveDateTime>,
    pub end_time: Option<NaiveDateTime>,
}
