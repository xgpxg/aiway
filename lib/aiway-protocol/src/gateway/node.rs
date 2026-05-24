//! # 网关节点信息
//!
//! 定义网关节点信息结构，用于接入层从控制台获取可用的网关节点。

use serde::{Deserialize, Serialize};

/// 网关节点信息，用于接入层转发
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayNodeInfo {
    /// 节点IP
    pub ip: String,
    /// 节点端口
    pub port: u16,
}

impl GatewayNodeInfo {
    /// 返回 ip:port 格式的地址
    pub fn addr(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}
