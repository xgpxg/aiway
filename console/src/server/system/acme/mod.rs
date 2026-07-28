use rocket::serde::{Deserialize, Serialize};

pub mod api;
mod request;
pub(crate) mod service;

/// DNS 提供商凭证
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DnsCredentials {
    Cloudflare {
        api_token: String,
    },
    Tencent {
        secret_id: String,
        secret_key: String,
    },
    Ali {
        access_key: String,
        secret_key: String,
    },
    Huawei {
        access_key: String,
        secret_key: String,
    },
    Volcengine {
        access_key: String,
        secret_key: String,
    },
    Baidu {
        access_key: String,
        secret_key: String,
    },
}

/// ACME 全局配置，存储在 system_config 表
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AcmeConfig {
    /// ACME 账户邮箱
    pub email: String,
    /// DNS 提供商：Cloudflare | Tencent | Ali | Huawei | Volcengine | Baidu
    pub dns_provider: String,
    /// DNS 提供商凭证
    pub dns_credentials: DnsCredentials,
    /// 是否使用 staging 环境
    pub staging: bool,
}

impl Default for DnsCredentials {
    fn default() -> Self {
        DnsCredentials::Cloudflare {
            api_token: String::new(),
        }
    }
}
