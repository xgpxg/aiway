use crate::common::constants::ENCRYPT_KEY;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashSet;
use std::fmt::{Debug, Display, Formatter};

/// 防火墙配置
///
/// 防火墙中不配置插件，因为插件需要获取请求上下文，而上下文是在安全校验后才提取的，在防火墙执行阶段无法获取。
/// 但是可以使用全局插件的方式在获取请求上下文后再校验。
#[derive(Clone, Serialize, Deserialize)]
pub struct Firewall {
    /// IP策略模式，allow或deny
    pub ip_policy_mode: AllowDenyPolicy,
    /// IP策略值，例如：192.168.1.1
    /// TODO 暂不支持网段，后面再支持
    pub ip_policy: HashSet<String>,
    /// 受信IP
    ///
    /// 受信IP将直接放行，不受访问策略的影响
    pub trust_ips: HashSet<String>,
    /// Referer策略模式，allow或deny
    pub referer_policy_mode: AllowDenyPolicy,
    /// Referer策略值，例如：https://aaa.com
    pub referer_policy: HashSet<String>,
    /// 是否允许空Referer
    pub allow_empty_referer: bool,
    /// 单个网关节点的最大连接数限制
    // /// 例如：127.0.0.1:8080/1000，
    // /// 对所有节点限制：*/2000，
    // /// 如果配置了具体的节点限制，则优先使用具体配置。
    pub max_connections: Option<usize>,
    /// API密钥的加密密钥，长度固定为32位，由控制台验证长度。
    /// 可能为空字符串，为空时使用默认密钥
    #[serde(
        default = "default_api_secret_encrypt_key",
        serialize_with = "serialize_encrypt_key",
        deserialize_with = "deserialize_encrypt_key"
    )]
    pub api_secret_encrypt_key: [u8; 32],
}

impl Default for Firewall {
    fn default() -> Self {
        Firewall {
            ip_policy_mode: AllowDenyPolicy::Disable,
            ip_policy: Default::default(),
            trust_ips: Default::default(),
            referer_policy_mode: Default::default(),
            referer_policy: Default::default(),
            allow_empty_referer: false,
            max_connections: Default::default(),
            api_secret_encrypt_key: ENCRYPT_KEY.clone(),
        }
    }
}

fn serialize_encrypt_key<S>(key: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let key_str = std::str::from_utf8(key).unwrap_or("");
    serializer.serialize_str(key_str)
}

fn deserialize_encrypt_key<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let mut key = [0u8; 32];

    if s.is_empty() {
        key = default_api_secret_encrypt_key();
    } else {
        let bytes = s.as_bytes();
        let len = std::cmp::min(32, bytes.len());
        key[..len].copy_from_slice(&bytes[..len]);
    }

    Ok(key)
}

fn default_api_secret_encrypt_key() -> [u8; 32] {
    ENCRYPT_KEY.clone()
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

impl Debug for Firewall {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Firewall")
            .field("ip_policy_mode", &self.ip_policy_mode)
            .field("ip_policy", &self.ip_policy)
            .field("trust_ips", &self.trust_ips)
            .field("referer_policy_mode", &self.referer_policy_mode)
            .field("referer_policy", &self.referer_policy)
            .field("allow_empty_referer", &self.allow_empty_referer)
            .field("max_connections", &self.max_connections)
            .field(
                "api_secret_encrypt_key",
                &format!(
                    "{}***",
                    String::from_utf8(self.api_secret_encrypt_key[0..5].to_vec()).unwrap()
                ),
            )
            .finish()
    }
}
