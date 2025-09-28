//! # 网关配置
//! 该配置为网关全局配置，对对所有网关节点生效
//!

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    /// 全局前置过滤器
    pub pre_filters: Vec<String>,
    /// 全局后置过滤器
    pub post_filters: Vec<String>,
}
