use serde::{Deserialize, Serialize};

/// 服务信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Service {
    /// 服务名
    pub name: String,
    /// 服务节点，支持域名或IP:PORT
    pub nodes: Vec<String>,
    /// 负载均衡策略
    #[serde(default = "LbStrategy::default")]
    pub lb: LbStrategy,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum LbStrategy {
    /// 随机
    #[serde(rename = "random")]
    #[default]
    Random,
    /// 轮询
    #[serde(rename = "random_robin")]
    RoundRobin,
}
