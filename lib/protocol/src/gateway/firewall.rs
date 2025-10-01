use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Firewall {
    /// IP策略模式，allow或deny
    pub ip_policy_mode: AllowDenyPolicy,
    /// IP策略值，例如：192.168.1.1
    /// TODO 暂不支持网段，后面再支持
    pub ip_policy: HashSet<String>,
    /// Referer策略模式，allow或deny
    pub referer_policy_mode: AllowDenyPolicy,
    /// Referer策略值，例如：https://aaa.com
    pub referer_policy: HashSet<String>,
    /// 是否允许空Referer
    pub allow_empty_referer: bool,
    /// 单个网关节点的最大连接数限制，
    /// 例如：127.0.0.1:8080/1000，
    /// 对所有节点限制：*/2000，
    /// 如果配置了具体的节点限制，则优先使用具体配置。
    pub max_connections: HashMap<String, usize>,
}

#[derive(Debug, Clone, Default, Eq, Ord, PartialOrd, PartialEq, Serialize, Deserialize)]
pub enum AllowDenyPolicy {
    /// 不启用该功能
    #[default]
    Disable,
    /// 允许
    Allow,
    /// 拒绝
    Deny,
}

impl From<&str> for AllowDenyPolicy {
    fn from(value: &str) -> Self {
        match value {
            "allow" => AllowDenyPolicy::Allow,
            "deny" => AllowDenyPolicy::Deny,
            _ => panic!("invalid allow deny policy"),
        }
    }
}
