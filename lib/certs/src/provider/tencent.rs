use async_trait::async_trait;
use hmac::{Hmac, Mac};
use reqwest::Client;
use sha2::Digest;
use sha2::Sha256;

use crate::provider::DnsProviderOps;
use crate::CertError;

/// 腾讯云 DNSPod API 3.0 实现（TC3-HMAC-SHA256 签名）
pub struct TencentProvider {
    secret_id: String,
    secret_key: String,
    client: Client,
}

impl TencentProvider {
    pub fn new(secret_id: String, secret_key: String) -> Self {
        Self {
            secret_id,
            secret_key,
            client: Client::new(),
        }
    }

    fn split_domain(domain: &str) -> (String, String) {
        let parts: Vec<&str> = domain.split('.').collect();
        if parts.len() < 2 {
            return (domain.to_string(), String::new());
        }
        let zone = parts[parts.len() - 2..].join(".");
        let sub = parts[..parts.len() - 2].join(".");
        (zone, sub)
    }

    async fn get_txt_record_id(&self, zone: &str, sub: &str) -> Result<Option<String>, CertError> {
        let subdomain = if sub.is_empty() {
            "_acme-challenge".to_string()
        } else {
            format!("_acme-challenge.{}", sub)
        };
        let body = serde_json::json!({
            "Domain": zone,
            "Subdomain": subdomain,
            "RecordType": "TXT",
        });
        let resp = self.call_api("DescribeRecordList", &body).await?;
        Ok(resp["Response"]["RecordList"]
            .as_array()
            .and_then(|list| list.first())
            .and_then(|r| r["RecordId"].as_i64())
            .map(|id| id.to_string()))
    }

    async fn call_api(
        &self,
        action: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, CertError> {
        let host = "dnspod.tencentcloudapi.com";
        let service = "dnspod";
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let date = format_timestamp_date(timestamp);

        let payload = serde_json::to_string(body).map_err(|e| CertError::DnsProvider {
            provider: self.name(),
            detail: format!("Failed to serialize body: {}", e),
        })?;
        let payload_hash = sha256_hex(&payload);

        let signed_headers = "content-type;host";
        let canonical_request = format!(
            "POST\n/\n\ncontent-type:application/json\nhost:{}\n{}\n{}",
            host, signed_headers, payload_hash
        );
        let canonical_request_hash = sha256_hex(&canonical_request);

        let credential_scope = format!("{}/{}/tc3_request", date, service);
        let string_to_sign = format!(
            "TC3-HMAC-SHA256\n{}\n{}\n{}",
            timestamp, credential_scope, canonical_request_hash
        );

        let secret_date = hmac_sha256(format!("TC3{}", self.secret_key).as_bytes(), &date);
        let secret_service = hmac_sha256(&secret_date, service);
        let secret_signing = hmac_sha256(&secret_service, "tc3_request");
        let signature = hex::encode(hmac_sha256(&secret_signing, &string_to_sign));

        let authorization = format!(
            "TC3-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
            self.secret_id, credential_scope, signed_headers, signature
        );

        let url = format!("https://{}/", host);
        let response = self
            .client
            .post(&url)
            .header("Authorization", &authorization)
            .header("Content-Type", "application/json")
            .header("Host", host)
            .header("X-TC-Action", action)
            .header("X-TC-Timestamp", timestamp.to_string())
            .header("X-TC-Version", "2021-03-23")
            .header("X-TC-Region", "ap-guangzhou")
            .json(body)
            .send()
            .await?;

        let resp: serde_json::Value = response.json().await?;

        if resp.get("Response").and_then(|r| r.get("Error")).is_some() {
            let err = &resp["Response"]["Error"];
            return Err(CertError::DnsProvider {
                provider: self.name(),
                detail: format!("[{}] {}", err["Code"], err["Message"]),
            });
        }

        Ok(resp)
    }
}

#[async_trait]
impl DnsProviderOps for TencentProvider {
    fn name(&self) -> &'static str {
        "tencent"
    }

    async fn create_txt(&self, domain: &str, value: &str) -> Result<(), CertError> {
        let (zone, sub) = Self::split_domain(domain);
        let sub_domain = if sub.is_empty() {
            "_acme-challenge".to_string()
        } else {
            format!("_acme-challenge.{}", sub)
        };

        let body = serde_json::json!({
            "Domain": zone,
            "SubDomain": sub_domain,
            "RecordType": "TXT",
            "RecordLine": "默认",
            "Value": value,
        });
        self.call_api("CreateRecord", &body).await?;
        Ok(())
    }

    async fn delete_txt(&self, domain: &str, _value: &str) -> Result<(), CertError> {
        let (zone, sub) = Self::split_domain(domain);
        if let Some(record_id) = self.get_txt_record_id(&zone, &sub).await? {
            let body = serde_json::json!({
                "Domain": zone,
                "RecordId": record_id.parse::<i64>().unwrap_or(0),
            });
            self.call_api("DeleteRecord", &body).await?;
        }
        Ok(())
    }
}

// -- 签名辅助函数 --

fn sha256_hex(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    hex::encode(hasher.finalize())
}

fn hmac_sha256(key: &[u8], data: &str) -> Vec<u8> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC key length ok");
    mac.update(data.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

fn format_timestamp_date(timestamp: u64) -> String {
    use chrono::TimeZone;
    chrono::Utc
        .timestamp_opt(timestamp as i64, 0)
        .unwrap()
        .format("%Y-%m-%d")
        .to_string()
}
