use async_trait::async_trait;
use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::Client;
use sha2::{Digest, Sha256};

use crate::CertError;
use crate::provider::DnsProviderOps;

type HmacSha256 = Hmac<Sha256>;

/// 百度云 DNS API v2 实现（BCE 签名）
pub struct BaiduProvider {
    access_key: String,
    secret_key: String,
    client: Client,
}

impl BaiduProvider {
    pub fn new(access_key: String, secret_key: String) -> Self {
        Self {
            access_key,
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

    /// 逐级查找 zone ID
    async fn get_zone_id(&self, domain: &str) -> Result<String, CertError> {
        let parts: Vec<&str> = domain.split('.').collect();
        for i in 0..parts.len().saturating_sub(1) {
            let name = parts[i..].join(".");
            let resp = self
                .call_api("GET", &format!("/v2/zone?name={}", url_encode(&name)), None)
                .await?;
            if let Some(zones) = resp["zones"].as_array()
                && let Some(zone) = zones.first()
                && let Some(id) = zone["id"].as_str()
            {
                return Ok(id.to_string());
            }
        }
        Err(CertError::DnsProvider {
            provider: self.name(),
            detail: format!("Zone not found for domain: {}", domain),
        })
    }

    /// 获取 TXT 记录 ID
    async fn get_record_id(&self, zone_id: &str, name: &str) -> Result<Option<String>, CertError> {
        let resp = self
            .call_api(
                "GET",
                &format!(
                    "/v2/zone/{}/record?name={}&type=TXT",
                    zone_id,
                    url_encode(name)
                ),
                None,
            )
            .await?;
        Ok(resp["records"]
            .as_array()
            .and_then(|r| r.first())
            .and_then(|r| r["id"].as_str())
            .map(|s| s.to_string()))
    }

    /// 带 BCE 签名的 API 调用
    async fn call_api(
        &self,
        method: &str,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, CertError> {
        let host = "dns.baidubce.com";
        let now = Utc::now();
        let timestamp = now.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let expire = 1800;

        let uri = path.split('?').next().unwrap_or(path);
        let query_string = path.split('?').nth(1).unwrap_or("");

        let payload = body
            .as_ref()
            .map(|b| serde_json::to_string(b).unwrap_or_default())
            .unwrap_or_default();
        let payload_hash = hex::encode(Sha256::digest(payload.as_bytes()));

        // 1. Auth String & Signing Key
        let auth_string = format!("bce-auth-v1/{}/{}/{}/", self.access_key, timestamp, expire);
        let signing_key = hmac_sha256(self.secret_key.as_bytes(), &auth_string);

        // 2. Canonical Request
        let signed_headers = "host;x-bce-date";
        let canonical_headers = format!("host:{}\nx-bce-date:{}\n", host, timestamp);
        let canonical_query = sort_query(query_string);
        let canonical_request = format!(
            "{}\n{}\n{}\n{}\n{}",
            method.to_uppercase(),
            uri,
            canonical_query,
            canonical_headers,
            signed_headers,
        );
        let signature = hex::encode(hmac_sha256(&signing_key, &canonical_request));

        let authorization = format!("{}{}/{}", auth_string, signed_headers, signature,);

        let url = format!("https://{}{}", host, path);
        let mut req = self
            .client
            .request(
                reqwest::Method::from_bytes(method.as_bytes()).map_err(|_| {
                    CertError::InvalidArgument(format!("Invalid HTTP method: {}", method))
                })?,
                &url,
            )
            .header("Host", host)
            .header("x-bce-date", &timestamp)
            .header("x-bce-content-sha256", &payload_hash)
            .header("Authorization", &authorization);

        if method == "POST" || method == "PUT" {
            req = req.header("Content-Type", "application/json; charset=UTF-8");
            if let Some(b) = body {
                req = req.json(&b);
            }
        }

        let resp = req.send().await?;
        let status = resp.status();
        let result: serde_json::Value = resp.json().await?;

        if !status.is_success() {
            let code = result["code"].as_str().unwrap_or("Unknown");
            let msg = result["message"].as_str().unwrap_or("unknown error");
            return Err(CertError::DnsProvider {
                provider: self.name(),
                detail: format!("[{}] {}: {}", status, code, msg),
            });
        }

        Ok(result)
    }
}

#[async_trait]
impl DnsProviderOps for BaiduProvider {
    fn name(&self) -> &'static str {
        "baidu"
    }

    async fn create_txt(&self, domain: &str, value: &str) -> Result<(), CertError> {
        let zone_id = self.get_zone_id(domain).await?;
        let (zone, sub) = Self::split_domain(domain);
        let name = if sub.is_empty() {
            format!("_acme-challenge.{}", zone)
        } else {
            format!("_acme-challenge.{}.{}", sub, zone)
        };

        let body = serde_json::json!({
            "name": name,
            "type": "TXT",
            "ttl": 60,
            "rdata": value,
            "zoneId": zone_id,
        });

        self.call_api("POST", &format!("/v2/zone/{}/record", zone_id), Some(body))
            .await?;
        Ok(())
    }

    async fn delete_txt(&self, domain: &str, _value: &str) -> Result<(), CertError> {
        let zone_id = self.get_zone_id(domain).await?;
        let (zone, sub) = Self::split_domain(domain);
        let name = if sub.is_empty() {
            format!("_acme-challenge.{}", zone)
        } else {
            format!("_acme-challenge.{}.{}", sub, zone)
        };

        if let Some(record_id) = self.get_record_id(&zone_id, &name).await? {
            self.call_api(
                "DELETE",
                &format!("/v2/zone/{}/record/{}", zone_id, record_id),
                None,
            )
            .await?;
        }
        Ok(())
    }
}

// -- 辅助函数 --

fn hmac_sha256(key: &[u8], data: &str) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC key length ok");
    mac.update(data.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

fn url_encode(s: &str) -> String {
    let mut result = String::new();
    for byte in s.as_bytes() {
        match byte {
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

/// 对查询字符串按键排序（BCE 签名要求）
fn sort_query(query: &str) -> String {
    let mut pairs: Vec<&str> = query.split('&').collect();
    pairs.sort();
    pairs.join("&")
}
