use std::sync::Arc;
use std::time::Duration;

use acme2::gen_rsa_private_key;
use acme2::{Account, AccountBuilder, Challenge, Csr, DirectoryBuilder, Order, OrderBuilder};
use openssl::pkey::PKey;

use crate::CertError;

const LETS_ENCRYPT_DIRECTORY: &str = "https://acme-v02.api.letsencrypt.org/directory";
pub const LETS_ENCRYPT_STAGING_DIRECTORY: &str =
    "https://acme-staging-v02.api.letsencrypt.org/directory";

const POLL_INTERVAL: Duration = Duration::from_secs(5);
const POLL_ATTEMPTS: usize = 12;

/// ACME 客户端，封装与 Let's Encrypt 的交互
pub struct AcmeClient {
    account: Arc<Account>,
}

/// DNS-01 挑战信息，包含域名和对应的 TXT 记录值
pub struct DnsChallenge {
    domain: String,
    /// TXT 记录值，需要写入 _acme-challenge.<domain>
    txt_value: String,
    challenge: Challenge,
}

/// 订单句柄，在 DNS 记录就绪后用于完成验证和下载证书
pub struct OrderHandle {
    order: Order,
    challenges: Vec<DnsChallenge>,
    private_key: PKey<openssl::pkey::Private>,
    private_key_pem: String,
}

impl AcmeClient {
    /// 创建新 ACME 账户（首次使用）
    pub async fn new(account_email: &str, staging: bool) -> Result<Self, CertError> {
        let dir_url = if staging {
            LETS_ENCRYPT_STAGING_DIRECTORY
        } else {
            LETS_ENCRYPT_DIRECTORY
        };
        tracing::info!("Using ACME directory: {}", dir_url);
        let dir = DirectoryBuilder::new(dir_url.to_string()).build().await?;

        let mut builder = AccountBuilder::new(dir);
        builder.contact(vec![format!("mailto:{}", account_email)]);
        builder.terms_of_service_agreed(true);
        let account = builder.build().await?;

        tracing::info!("ACME account created: {:?}", account.status);
        Ok(Self { account })
    }

    /// 恢复已有 ACME 账户（传入之前持久化的私钥 PEM）
    pub async fn from_key(
        account_email: &str,
        staging: bool,
        account_key_pem: &str,
    ) -> Result<Self, CertError> {
        let dir_url = if staging {
            LETS_ENCRYPT_STAGING_DIRECTORY
        } else {
            LETS_ENCRYPT_DIRECTORY
        };
        tracing::info!("Using ACME directory: {}", dir_url);

        let dir = DirectoryBuilder::new(dir_url.to_string()).build().await?;

        let private_key = PKey::private_key_from_pem(account_key_pem.as_bytes())?;

        let mut builder = AccountBuilder::new(dir);
        builder.contact(vec![format!("mailto:{}", account_email)]);
        builder.terms_of_service_agreed(true);
        builder.private_key(private_key);
        let account = builder.build().await?;

        tracing::info!("ACME account restored: {:?}", account.status);
        Ok(Self { account })
    }

    /// 获取账户私钥 PEM，用于持久化
    pub fn account_key_pem(&self) -> Result<String, CertError> {
        let key = self.account.private_key();
        let pem = key.private_key_to_pem_pkcs8()?;
        Ok(String::from_utf8(pem)?)
    }

    /// 准备订单：创建订单、生成密钥对、获取 DNS-01 挑战列表
    ///
    /// 返回 (OrderHandle, Vec<(domain, txt_value)>)，
    /// 调用方根据 domain 和 txt_value 创建 DNS TXT 记录。
    pub async fn prepare_order(
        &self,
        domains: &[&str],
    ) -> Result<(OrderHandle, Vec<(String, String)>), CertError> {
        let mut builder = OrderBuilder::new(self.account.clone());
        for domain in domains {
            builder.add_dns_identifier(domain.to_string());
        }
        let order = builder.build().await?;
        tracing::info!("Order created: {:?}", order.status);

        let authorizations = order.authorizations().await?;
        let mut challenges = Vec::new();
        for auth in authorizations {
            let domain = auth.identifier.value.clone();

            let challenge = auth.get_challenge("dns-01").ok_or_else(|| {
                CertError::InvalidArgument(format!(
                    "DNS-01 challenge not available for domain: {}",
                    domain
                ))
            })?;

            let txt_value = challenge.key_authorization_encoded()?.ok_or_else(|| {
                CertError::InvalidArgument(format!(
                    "Failed to get DNS-01 challenge value for domain: {}",
                    domain
                ))
            })?;

            challenges.push(DnsChallenge {
                domain,
                txt_value,
                challenge,
            });
        }
        tracing::info!("Got {} DNS-01 challenges", challenges.len());

        let challenge_info: Vec<_> = challenges
            .iter()
            .map(|c| (c.domain.clone(), c.txt_value.clone()))
            .collect();

        let private_key = gen_rsa_private_key(4096)?;
        let private_key_pem = String::from_utf8(private_key.private_key_to_pem_pkcs8()?)?;

        let handle = OrderHandle {
            order,
            challenges,
            private_key,
            private_key_pem,
        };

        Ok((handle, challenge_info))
    }

    /// 完成订单：验证所有挑战、最终化订单、下载证书
    pub async fn complete_order(handle: OrderHandle) -> Result<CertBundle, CertError> {
        for c in &handle.challenges {
            tracing::info!("Validating challenge for domain: {}", c.domain);
            let challenge = c.challenge.validate().await?;
            challenge.wait_done(POLL_INTERVAL, POLL_ATTEMPTS).await?;
            tracing::info!("Challenge validated for domain: {}", c.domain);
        }

        let auths = handle.order.authorizations().await?;
        for auth in auths {
            auth.wait_done(POLL_INTERVAL, POLL_ATTEMPTS).await?;
        }

        let order = handle
            .order
            .wait_ready(POLL_INTERVAL, POLL_ATTEMPTS)
            .await?;
        let order = order.finalize(Csr::Automatic(handle.private_key)).await?;
        let order = order.wait_done(POLL_INTERVAL, POLL_ATTEMPTS).await?;

        let cert_chain = order
            .certificate()
            .await?
            .ok_or_else(|| CertError::InvalidArgument("No certificate returned".to_string()))?;

        // cert_chain 是 Vec<openssl::x509::X509>，每个元素是一个证书对象
        let mut cert_pem = String::new();
        for cert in &cert_chain {
            let pem_bytes = cert.to_pem()?;
            cert_pem.push_str(&String::from_utf8(pem_bytes)?);
        }

        // 解析证书过期时间
        // openssl 的 not_after() 输出格式如 "Jul 27 18:00:00 2026 GMT"
        let expires_at = if let Some(leaf) = cert_chain.first() {
            let date_str = leaf.not_after().to_string();
            chrono::NaiveDateTime::parse_from_str(date_str.trim(), "%b %d %H:%M:%S %Y %Z")
                .map(|dt| dt.and_utc().timestamp())
                .unwrap_or(0)
        } else {
            0
        };

        Ok(CertBundle {
            cert_pem,
            key_pem: handle.private_key_pem,
            expires_at,
        })
    }
}

/// 证书签发结果
pub struct CertBundle {
    pub cert_pem: String,
    pub key_pem: String,
    pub expires_at: i64,
}
