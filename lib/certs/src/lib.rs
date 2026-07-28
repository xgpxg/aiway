use std::time::Duration;

use crate::acme::AcmeClient;
use crate::provider::DnsProviderOps;

mod acme;
mod error;
mod provider;

pub use error::CertError;

/// DNS 提供商配置
pub enum DnsProvider {
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

impl DnsProvider {
    fn create_provider(&self) -> Box<dyn DnsProviderOps> {
        match self {
            DnsProvider::Cloudflare { api_token } => Box::new(
                provider::cloudflare::CloudflareProvider::new(api_token.clone()),
            ),
            DnsProvider::Tencent {
                secret_id,
                secret_key,
            } => Box::new(provider::tencent::TencentProvider::new(
                secret_id.clone(),
                secret_key.clone(),
            )),
            DnsProvider::Ali {
                access_key,
                secret_key,
            } => Box::new(provider::ali::AliProvider::new(
                access_key.clone(),
                secret_key.clone(),
            )),
            DnsProvider::Huawei {
                access_key,
                secret_key,
            } => Box::new(provider::huawei::HuaweiProvider::new(
                access_key.clone(),
                secret_key.clone(),
            )),
            DnsProvider::Volcengine {
                access_key,
                secret_key,
            } => Box::new(provider::volc::VolcProvider::new(
                access_key.clone(),
                secret_key.clone(),
            )),
            DnsProvider::Baidu {
                access_key,
                secret_key,
            } => Box::new(provider::baidu::BaiduProvider::new(
                access_key.clone(),
                secret_key.clone(),
            )),
        }
    }
}

/// 证书签发结果
#[derive(Debug)]
pub struct CertResult {
    pub cert_pem: String,
    pub key_pem: String,
    pub expires_at: i64,
}

/// 证书签发器
pub struct CertIssuer {
    client: AcmeClient,
    account_key_pem: String,
}

impl CertIssuer {
    /// 创建新账户的证书签发器（首次使用）
    ///
    /// 账户私钥自动生成，可通过 `account_key_pem()` 获取并持久化。
    pub async fn new(account_email: &str, staging: bool) -> Result<Self, CertError> {
        let client = AcmeClient::new(account_email, staging).await?;
        let account_key_pem = client.account_key_pem()?;
        Ok(Self {
            client,
            account_key_pem,
        })
    }

    /// 恢复已有账户的证书签发器（传入已持久化的账户私钥 PEM）
    pub async fn from_key(
        account_email: &str,
        staging: bool,
        account_key_pem: &str,
    ) -> Result<Self, CertError> {
        let client = AcmeClient::from_key(account_email, staging, account_key_pem).await?;
        Ok(Self {
            client,
            account_key_pem: account_key_pem.to_string(),
        })
    }

    /// 获取账户私钥 PEM，供持久化使用
    pub fn account_key_pem(&self) -> &str {
        &self.account_key_pem
    }

    /// 为指定域名申请证书（支持通配符）
    ///
    /// - `domains` - 域名列表，如 `["example.com", "*.example.com"]`
    /// - `provider` - DNS 提供商配置
    /// - `dns_propagation_secs` - DNS 传播等待时间（秒），建议 10-30
    pub async fn issue(
        &self,
        domains: &[&str],
        provider: &DnsProvider,
        dns_propagation_secs: u64,
    ) -> Result<CertResult, CertError> {
        let dns = provider.create_provider();
        let propagation = Duration::from_secs(dns_propagation_secs);

        // 1. 准备订单：创建订单、获取challenge
        let (handle, challenges) = self.client.prepare_order(domains).await?;
        tracing::info!(
            "Order prepared, {} DNS challenges to create",
            challenges.len()
        );

        // 2. 创建 DNS TXT 记录
        for (domain, txt_value) in &challenges {
            tracing::info!("Creating TXT record: _acme-challenge.{}", domain);
            dns.create_txt(domain, txt_value).await?;
        }
        tracing::info!(
            "All TXT records created, waiting {}s for propagation",
            propagation.as_secs()
        );

        // 3. 等待 DNS 传播
        tokio::time::sleep(propagation).await;

        // 4. 完成验证和下载
        let result = AcmeClient::complete_order(handle).await;

        // 5. 无论如何都清理 DNS 记录
        for (domain, txt_value) in &challenges {
            if let Err(e) = dns.delete_txt(domain, txt_value).await {
                tracing::warn!("Failed to clean up TXT record for {}: {}", domain, e);
            }
        }

        let bundle = result?;

        Ok(CertResult {
            cert_pem: bundle.cert_pem,
            key_pem: bundle.key_pem,
            expires_at: bundle.expires_at,
        })
    }
}

mod tests {
    #[allow(unused)]
    use super::*;
    #[tokio::test]
    async fn test_issue() {
        let issuer = CertIssuer::new("1584929962@qq.com", true).await.unwrap();
        let key = std::env::var("ALI_KEY").expect("ALI_KEY not set");
        let secret = std::env::var("ALI_SECRET").expect("ALI_SECRET not set");
        let result = issuer
            .issue(
                &["cert-test.coderbox.cn"],
                &DnsProvider::Ali {
                    access_key: key,
                    secret_key: secret,
                },
                10,
            )
            .await;
        println!("{:?}", result);
        assert!(result.is_ok());
    }
}
