use serde::Deserialize;
use std::fmt::{Display, Formatter};

/// 插件配置
#[derive(Debug, Deserialize)]
pub struct Plugin {
    /// 插件名称
    pub name: String,
    /// 插件执行阶段
    pub phase: PluginPhase,
    /// 下载地址
    pub url: String,
    /// 插件版本
    pub version: String,
}
#[derive(Debug, Clone, Deserialize)]
pub enum PluginPhase {
    /// 全局，请求阶段执行
    #[serde(rename = "global-pre")]
    GlobalPre,
    /// 全局，响应阶段执行
    #[serde(rename = "global-post")]
    GlobalPost,
    /// 路由，请求阶段执行
    #[serde(rename = "pre")]
    Pre,
    /// 路由，响应阶段执行
    #[serde(rename = "post")]
    Post,
}
impl Display for PluginPhase {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginPhase::GlobalPre => write!(f, "global-pre"),
            PluginPhase::GlobalPost => write!(f, "global-post"),
            PluginPhase::Pre => write!(f, "pre"),
            PluginPhase::Post => write!(f, "post"),
        }
    }
}
