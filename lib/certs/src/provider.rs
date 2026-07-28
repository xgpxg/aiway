use async_trait::async_trait;

use crate::CertError;

/// DNS 提供商的操作接口
///
/// 每个提供商实现此 trait 来支持 ACME DNS-01 挑战的 TXT 记录管理。
#[async_trait]
pub trait DnsProviderOps: Send + Sync {
    /// 提供商名称，用于错误消息
    fn name(&self) -> &'static str;

    /// 添加 _acme-challenge.<domain> 的 TXT 记录
    ///
    /// `domain` 是完整的域名（如 "example.com"），
    /// `value` 是挑战值（由 ACME 服务器提供）。
    async fn create_txt(&self, domain: &str, value: &str) -> Result<(), CertError>;

    /// 验证完成后删除 TXT 记录
    async fn delete_txt(&self, domain: &str, value: &str) -> Result<(), CertError>;
}

pub(crate) mod ali;
pub(crate) mod baidu;
pub(crate) mod cloudflare;
pub(crate) mod tencent;
pub(crate) mod huawei;
pub(crate) mod volc;
