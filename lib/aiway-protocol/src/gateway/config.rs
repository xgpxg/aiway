use serde::{Deserialize, Serialize};

/// # 系统配置
///
/// 主要用于网关获取需要的配置内容。
/// 这些配置通常保存在数据库中，由网关节点定时同步。
/// 网关节点需要保存Config为全局实例。
#[derive(Debug, Serialize, Deserialize, PartialOrd, PartialEq)]
pub struct Config {
    // 暂时没有配置字段
}

impl Config {}
