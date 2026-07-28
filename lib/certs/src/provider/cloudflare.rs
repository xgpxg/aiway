use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use serde::Serialize;

use crate::CertError;
use crate::provider::DnsProviderOps;

/// Cloudflare DNS API 实现
pub struct CloudflareProvider {
    api_token: String,
    client: Client,
}

impl CloudflareProvider {
    pub fn new(api_token: String) -> Self {
        Self {
            api_token,
            client: Client::new(),
        }
    }

    /// 获取域名对应的 Zone ID
    /// 逐级尝试子域名，直到找到匹配的 zone
    /// 如 "a.b.example.com" → 先试 "a.b.example.com"，再试 "b.example.com"，再试 "example.com"
    async fn get_zone_id(&self, domain: &str) -> Result<String, CertError> {
        let parts: Vec<&str> = domain.split('.').collect();
        // 至少保留两级（example.com）
        for i in 0..parts.len().saturating_sub(1) {
            let candidate = parts[i..].join(".");
            let url = format!(
                "https://api.cloudflare.com/client/v4/zones?name={}",
                candidate
            );
            let resp = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.api_token))
                .send()
                .await?
                .json::<CfApiResponse<Vec<CfZone>>>()
                .await?;

            if resp.success
                && let Some(zones) = resp.result
                && let Some(zone) = zones.into_iter().next()
            {
                return Ok(zone.id);
            }
        }

        Err(CertError::DnsProvider {
            provider: self.name(),
            detail: format!("Zone not found for domain: {}", domain),
        })
    }

    async fn get_record_id(
        &self,
        zone_id: &str,
        record_name: &str,
    ) -> Result<Option<String>, CertError> {
        #[derive(Deserialize)]
        struct Record {
            id: String,
        }

        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records?type=TXT&name={}",
            zone_id, record_name
        );
        let resp = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .await?
            .json::<CfApiResponse<Vec<Record>>>()
            .await?;

        Ok(resp.result.and_then(|r| r.into_iter().next()).map(|r| r.id))
    }
}

#[async_trait]
impl DnsProviderOps for CloudflareProvider {
    fn name(&self) -> &'static str {
        "cloudflare"
    }

    async fn create_txt(&self, domain: &str, value: &str) -> Result<(), CertError> {
        let zone_id = self.get_zone_id(domain).await?;
        let record_name = format!("_acme-challenge.{}", domain);

        #[derive(Serialize)]
        struct CreateRecord {
            type_: String,
            name: String,
            content: String,
            ttl: u32,
        }

        let body = CreateRecord {
            type_: "TXT".to_string(),
            name: record_name,
            content: format!("\"{}\"", value),
            ttl: 60,
        };

        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            zone_id
        );
        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .json(&body)
            .send()
            .await?
            .json::<CfApiResponse<serde_json::Value>>()
            .await?;

        if !resp.success {
            return Err(CertError::DnsProvider {
                provider: self.name(),
                detail: resp
                    .errors
                    .first()
                    .map(|e| e.to_string())
                    .unwrap_or_default(),
            });
        }

        Ok(())
    }

    async fn delete_txt(&self, domain: &str, _value: &str) -> Result<(), CertError> {
        let zone_id = self.get_zone_id(domain).await?;
        let record_name = format!("_acme-challenge.{}", domain);

        if let Some(record_id) = self.get_record_id(&zone_id, &record_name).await? {
            let url = format!(
                "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
                zone_id, record_id
            );
            let resp = self
                .client
                .delete(&url)
                .header("Authorization", format!("Bearer {}", self.api_token))
                .send()
                .await?
                .json::<CfApiResponse<serde_json::Value>>()
                .await?;

            if !resp.success {
                return Err(CertError::DnsProvider {
                    provider: self.name(),
                    detail: resp
                        .errors
                        .first()
                        .map(|e| e.to_string())
                        .unwrap_or_default(),
                });
            }
        }

        Ok(())
    }
}

// -- Cloudflare API 响应类型 --

#[derive(Deserialize)]
struct CfApiResponse<T> {
    success: bool,
    result: Option<T>,
    errors: Vec<CfError>,
}

#[derive(Deserialize)]
struct CfError {
    code: u32,
    message: String,
}

impl std::fmt::Display for CfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct CfZone {
    id: String,
    name: String,
}
