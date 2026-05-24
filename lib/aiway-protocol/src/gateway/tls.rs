use serde::{Deserialize, Serialize};

/// 单个证书（兼容旧接口）
#[derive(Debug, Serialize, Deserialize)]
pub struct Cert {
    pub cert: Vec<u8>,
    pub key: Vec<u8>,
}

/// 证书条目，一个域名对应一个证书，用于 SNI 动态证书匹配
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertEntry {
    /// 绑定的域名，如 "example.com" 或 "*.example.com"
    pub domain: String,
    /// PEM 格式的证书内容
    pub cert: Vec<u8>,
    /// PEM 格式的私钥内容
    pub key: Vec<u8>,
}
