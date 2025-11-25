use derive_builder::Builder;
use rbatis::crud;
use rocket::serde::{Deserialize, Serialize};

/// 请求地区统计（小时级，保留近30天内的），当前小时区间内的每分钟更新一次
#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
#[builder(default)]
pub struct StatisticsRequestProvince {
    /// 省份
    pub province: Option<String>,
    ///  数量
    pub count: Option<i64>,
    /// 起始时间戳（秒），包含。
    /// 即小时区间的开始时间
    pub start_time: Option<i64>,
    /// 结束时间戳（秒），不包含
    /// 即小时区间的结束时间
    pub end_time: Option<i64>,
}

crud!(StatisticsRequestProvince {});
