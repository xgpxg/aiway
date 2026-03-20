//! # 全局过滤器
//! 该配置为全局配置，对所有网关节点生效
//!

use crate::gateway::plugin::ConfiguredPlugin;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalPlugin {
    /// 全局插件
    pub plugins: Vec<ConfiguredPlugin>,
}
