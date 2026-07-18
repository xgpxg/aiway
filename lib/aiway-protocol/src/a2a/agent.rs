use serde::{Deserialize, Serialize};

/// A2A Agent 定义
///
/// 描述一个可通过网关代理的 A2A Agent。
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct Agent {
    /// Agent 唯一标识名称
    pub name: String,
    /// Agent 描述
    pub description: Option<String>,
    /// Agent 端点 URL
    pub url: String,
}
