use serde::Deserialize;

/// 服务信息
#[derive(Debug, Default, Deserialize)]
pub struct Service {
    /// 服务ID
    pub id: String,
    /// 服务节点，支持域名或IP:PORT
    pub nodes: Vec<String>,
    #[serde(default = "LbStrategy::default")]
    pub lb: LbStrategy,
}
#[derive(Debug, Default, Clone, Deserialize)]
pub enum LbStrategy {
    #[serde(rename = "r")]
    #[default]
    Random,
    #[serde(rename = "rr")]
    RoundRobin,
}
