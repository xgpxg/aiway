use serde::{Deserialize, Serialize};

/// 服务信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Service {
    /// 服务名
    pub name: String,
    /// 服务节点，支持域名或IP:PORT
    pub nodes: Vec<String>,
    #[serde(default = "LbStrategy::default")]
    pub lb: LbStrategy,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum LbStrategy {
    #[serde(rename = "random")]
    #[default]
    Random,
    #[serde(rename = "random_robin")]
    RoundRobin,
}
