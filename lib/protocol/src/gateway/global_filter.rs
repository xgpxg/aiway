//! # 全局过滤器
//! 该配置为全局配置，对所有网关节点生效
//!

use crate::gateway::plugin::ConfiguredPlugin;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalFilter {
    /// 全局前置过滤器
    pub pre_filters: Vec<ConfiguredPlugin>,
    /// 全局后置过滤器
    pub post_filters: Vec<ConfiguredPlugin>,
}

impl Default for GlobalFilter {
    fn default() -> Self {
        GlobalFilter {
            pre_filters: vec![],
            post_filters: vec![],
        }
    }
}
