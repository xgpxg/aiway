use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionRequestCountReq {
    /// 起始时间戳（包含）
    pub start_time: Option<i64>,
    /// 结束时间戳（包含）
    pub end_time: Option<i64>,
}
