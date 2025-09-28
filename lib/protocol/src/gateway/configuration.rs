//! # 网关配置
//! 该配置为网关全局配置，对对所有网关节点生效
//!

use serde::{Deserialize, Serialize};
use crate::gateway::plugin::ConfiguredPlugin;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    /// 全局前置过滤器
    pub pre_filters: Vec<ConfiguredPlugin>,
    /// 全局后置过滤器
    pub post_filters: Vec<ConfiguredPlugin>,
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            pre_filters: vec![],
            post_filters: vec![],
        }
    }
}