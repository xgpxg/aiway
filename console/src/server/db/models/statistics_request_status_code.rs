use crate::server::metrics::RequestStatusCountReq;
use derive_builder::Builder;
use rbatis::executor::Executor;
use rbatis::{crud, htmlsql};
use rocket::serde::{Deserialize, Serialize};

/// 请求状态码统计（分钟级，保留近1年的）
#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
#[builder(default)]
pub struct StatisticsRequestStatusCode {
    /// 状态码
    pub status_code: Option<u16>,
    ///  数量
    pub count: Option<i64>,
    /// 分钟起始时间戳（秒，0秒），包含，范围为`[state_time, state_time+59]`
    pub state_time: Option<i64>,
}

crud!(StatisticsRequestStatusCode {});
htmlsql!(status_code_request_count(rb: &dyn Executor, param :&RequestStatusCountReq)  -> Vec<StatisticsRequestStatusCode> => "src/server/db/mapper/statistics_request_status_code.html");
