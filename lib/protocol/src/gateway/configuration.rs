//! # 网关配置
//! 该配置为网关全局配置，对对所有网关节点生效
//!

use crate::gateway::plugin::ConfiguredPlugin;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    // /// 防火墙配置 - 已经独立出来
    // pub firewall: Firewall,
    /// 全局前置过滤器
    pub pre_filters: Vec<ConfiguredPlugin>,
    /// 全局后置过滤器
    pub post_filters: Vec<ConfiguredPlugin>,
    // TODO 个性化配置

    // TODO 通知和提醒配置
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            // firewall: Firewall::default(),
            pre_filters: vec![],
            post_filters: vec![],
        }
    }
}
