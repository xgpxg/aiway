use serde::{Deserialize, Serialize};

/// 服务信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Service {
    /// 服务名
    pub name: String,
    /// 服务节点，支持域名或IP:PORT
    pub nodes: Vec<ServiceNode>,
    /// 负载均衡策略
    #[serde(default = "LbStrategy::default")]
    pub lb: LbStrategy,
}

/// 服务节点
///
/// 携带权重信息，供加权类负载均衡策略使用。
/// 反序列化同时兼容两种 JSON 形态，保证老数据（纯字符串地址）无需迁移即可读取：
/// - `"10.0.0.1:8080"` → `{ addr: "10.0.0.1:8080", weight: 1 }`
/// - `{ "addr": "10.0.0.1:8080", "weight": 3 }`
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServiceNode {
    /// 节点地址，支持域名或IP:PORT
    pub addr: String,
    /// 权重，最小为 1，默认 1
    #[serde(default = "default_weight")]
    pub weight: u32,
}

fn default_weight() -> u32 {
    1
}

impl ServiceNode {
    /// 仅指定地址构造节点，权重默认为 1
    pub fn new(addr: impl Into<String>) -> Self {
        Self {
            addr: addr.into(),
            weight: 1,
        }
    }

    /// 指定地址与权重构造节点
    pub fn with_weight(addr: impl Into<String>, weight: u32) -> Self {
        Self {
            addr: addr.into(),
            weight: weight.max(1),
        }
    }
}

impl<'de> Deserialize<'de> for ServiceNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr {
            Simple(String),
            Full {
                addr: String,
                #[serde(default = "default_weight")]
                weight: u32,
            },
        }

        Ok(match Repr::deserialize(deserializer)? {
            Repr::Simple(addr) => ServiceNode { addr, weight: 1 },
            Repr::Full { addr, weight } => ServiceNode {
                addr,
                weight: weight.max(1),
            },
        })
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LbStrategy {
    /// 随机
    #[default]
    Random,
    /// 轮询
    RoundRobin,
    /// 加权轮询（平滑加权轮询）
    WeightedRoundRobin,
    /// 加权随机
    WeightedRandom,
    /// 最少连接
    LeastConn,
    /// IP 哈希（会话保持，按客户端真实 IP）
    IpHash,
    // TODO 考虑一个Custom策略，由用户自定义需要的Key组合，再通过Hash环选择节点
}
