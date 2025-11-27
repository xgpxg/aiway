use crate::server::metrics::{RegionRequestCountReq, RegionRequestCountRes};
use derive_builder::Builder;
use rbatis::executor::Executor;
use rbatis::{crud, htmlsql};
use rocket::serde::{Deserialize, Serialize};

/// 请求地区统计（小时级，保留近1年的），当前小时区间内的每分钟更新一次
#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
#[builder(default)]
pub struct StatisticsRequestProvince {
    /// 省份
    pub province: Option<String>,
    ///  数量
    pub count: Option<i64>,
    /// 起始时间戳（秒，0分0秒），包含。
    pub start_time: Option<i64>,
    /// 结束时间戳（秒，59分59秒），包含
    /// 即小时区间的结束时间
    pub end_time: Option<i64>,
}

crud!(StatisticsRequestProvince {});
htmlsql!(region_request_count(rb: &dyn Executor, param :&RegionRequestCountReq)  -> Vec<RegionRequestCountRes> => "src/server/db/mapper/statistics_request_province.html");
