use async_trait::async_trait;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use hmac::{Hmac, Mac};
use reqwest::Client;
use sha1::Sha1;

use crate::provider::DnsProviderOps;
use crate::CertError;

/// 阿里云 DNS API 实现
pub struct AliProvider {
    access_key: String,
    secret_key: String,
    client: Client,
}

impl AliProvider {
    pub fn new(access_key: String, secret_key: String) -> Self {
        Self {
            access_key,
            secret_key,
            client: Client::new(),
        }
    }

    /// 从完整域名中解析 (zone, subdomain)
    fn split_domain(domain: &str) -> (String, String) {
        let parts: Vec<&str> = domain.split('.').collect();
        if parts.len() < 2 {
            return (domain.to_string(), String::new());
        }
        let zone = parts[parts.len() - 2..].join(".");
        let sub = parts[..parts.len() - 2].join(".");
        (zone, sub)
    }

    async fn call_api(&self, params: &[(&str, &str)]) -> Result<serde_json::Value, CertError> {
        let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let nonce = format!(
            "{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );

        let mut all_params: Vec<(&str, &str)> = vec![
            ("Format", "json"),
            ("Version", "2015-01-09"),
            ("AccessKeyId", &self.access_key),
            ("SignatureMethod", "HMAC-SHA1"),
            ("Timestamp", &timestamp),
            ("SignatureVersion", "1.0"),
            ("SignatureNonce", &nonce),
        ];
        all_params.extend_from_slice(params);
        all_params.sort_by(|a, b| a.0.cmp(b.0));

        let canonicalized_query: String = all_params
            .iter()
            .map(|(k, v)| format!("{}={}", percent_encode(k), percent_encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        let string_to_sign = format!(
            "GET&{}&{}",
            percent_encode("/"),
            percent_encode(&canonicalized_query)
        );

        let signing_key = format!("{}&", self.secret_key);
        let mut mac = Hmac::<Sha1>::new_from_slice(signing_key.as_bytes()).map_err(|e| {
            CertError::DnsProvider {
                provider: self.name(),
                detail: format!("HMAC init error: {}", e),
            }
        })?;
        mac.update(string_to_sign.as_bytes());
        let signature = BASE64.encode(mac.finalize().into_bytes());

        let url = format!(
            "https://alidns.aliyuncs.com/?{}&Signature={}",
            canonicalized_query,
            percent_encode(&signature)
        );

        let resp = self.client.get(&url).send().await?;
        let status = resp.status();
        let body: serde_json::Value = resp.json().await?;

        if !status.is_success() || body.get("Code").is_some() {
            let code = body.get("Code").and_then(|c| c.as_str()).unwrap_or("");
            let msg = body.get("Message").and_then(|m| m.as_str()).unwrap_or("");
            return Err(CertError::DnsProvider {
                provider: self.name(),
                detail: format!("[{}] {}", code, msg),
            });
        }

        Ok(body)
    }

    async fn get_record_id(&self, zone: &str, rr: &str) -> Result<Option<String>, CertError> {
        let sub_domain = format!("{}.{}", rr, zone);
        let resp = self
            .call_api(&[
                ("Action", "DescribeSubDomainRecords"),
                ("DomainName", zone),
                ("SubDomain", &sub_domain),
                ("Type", "TXT"),
            ])
            .await?;

        Ok(resp["DomainRecords"]["Record"]
            .as_array()
            .and_then(|records| records.first())
            .and_then(|r| r["RecordId"].as_str())
            .map(|s| s.to_string()))
    }
}

#[async_trait]
impl DnsProviderOps for AliProvider {
    fn name(&self) -> &'static str {
        "aliyun"
    }

    async fn create_txt(&self, domain: &str, value: &str) -> Result<(), CertError> {
        let (zone, sub) = Self::split_domain(domain);
        let rr = if sub.is_empty() {
            "_acme-challenge".to_string()
        } else {
            format!("_acme-challenge.{}", sub)
        };

        self.call_api(&[
            ("Action", "AddDomainRecord"),
            ("DomainName", &zone),
            ("RR", &rr),
            ("Type", "TXT"),
            ("Value", value),
        ])
        .await?;
        Ok(())
    }

    async fn delete_txt(&self, domain: &str, _value: &str) -> Result<(), CertError> {
        let (zone, sub) = Self::split_domain(domain);
        let rr = if sub.is_empty() {
            "_acme-challenge".to_string()
        } else {
            format!("_acme-challenge.{}", sub)
        };

        if let Some(record_id) = self.get_record_id(&zone, &rr).await? {
            self.call_api(&[("Action", "DeleteDomainRecord"), ("RecordId", &record_id)])
                .await?;
        }
        Ok(())
    }
}

/// 阿里云 API 的百分号编码
fn percent_encode(s: &str) -> String {
    let mut result = String::new();
    for byte in s.as_bytes() {
        match *byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(*byte as char);
            }
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}
