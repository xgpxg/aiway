use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionRequestCountReq {
    /// 起始时间戳（包含），小时开始时间，0分0秒
    pub start_timestamp: Option<i64>,
    /// 结束时间戳（包含），小时结束时间，59分59秒
    pub end_timestamp: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestStatusCountReq {
    /// 起始时间戳（包含），分钟开始时间，0秒
    pub start_timestamp: Option<i64>,
    /// 结束时间戳（包含），分钟结束时间，59秒
    pub end_timestamp: Option<i64>,
}
