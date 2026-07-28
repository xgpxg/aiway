use async_trait::async_trait;
use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::Client;
use sha2::{Digest, Sha256};

use crate::CertError;
use crate::provider::DnsProviderOps;

type HmacSha256 = Hmac<Sha256>;

/// 华为云 DNS API v2 实现（SDK-HMAC-SHA256 签名）
pub struct HuaweiProvider {
    access_key: String,
    secret_key: String,
    client: Client,
}

impl HuaweiProvider {
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

    /// 逐级查找 zone ID（带尾部点）
    async fn get_zone_id(&self, domain: &str) -> Result<String, CertError> {
        let parts: Vec<&str> = domain.split('.').collect();
        for i in 0..parts.len().saturating_sub(1) {
            let name = format!("{}.", parts[i..].join("."));
            let resp = self
                .call_api(
                    "GET",
                    &format!("/v2/zones?name={}&limit=1", url_encode(&name)),
                    None,
                )
                .await?;
            if let Some(zone) = resp["zones"].as_array().and_then(|z| z.first())
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

    /// 获取 TXT 记录集 ID
    async fn get_recordset_id(
        &self,
        zone_id: &str,
        name: &str,
    ) -> Result<Option<String>, CertError> {
        let resp = self
            .call_api(
                "GET",
                &format!(
                    "/v2/zones/{}/recordsets?name={}&type=TXT&limit=1",
                    zone_id,
                    url_encode(name)
                ),
                None,
            )
            .await?;
        Ok(resp["recordsets"]
            .as_array()
            .and_then(|r| r.first())
            .and_then(|r| r["id"].as_str())
            .map(|s| s.to_string()))
    }

    /// 带 SDK-HMAC-SHA256 签名的 API 调用
    async fn call_api(
        &self,
        method: &str,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, CertError> {
        let host = "dns.myhuaweicloud.com";
        let region = "cn-north-4";
        let service = "dns";
        let now = Utc::now();
        let date = now.format("%Y%m%d").to_string();
        let datetime = now.format("%Y%m%dT%H%M%SZ").to_string();

        let payload = body
            .as_ref()
            .map(|b| serde_json::to_string(b).unwrap_or_default())
            .unwrap_or_default();
        let payload_hash = hex::encode(Sha256::digest(payload.as_bytes()));

        let uri = path.split('?').next().unwrap_or(path);
        let query_string = path.split('?').nth(1).unwrap_or("");

        // 1. Canonical Request
        let signed_headers = "host;x-sdk-date";
        let canonical_headers = format!("host:{}\nx-sdk-date:{}\n", host, datetime);
        let canonical_request = format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            method.to_uppercase(),
            uri,
            query_string,
            canonical_headers,
            signed_headers,
            payload_hash,
        );
        let canonical_request_hash = hex::encode(Sha256::digest(canonical_request.as_bytes()));

        // 2. String to Sign
        let credential_scope = format!("{}/{}/{}/aws4_request", date, region, service);
        let string_to_sign = format!(
            "SDK-HMAC-SHA256\n{}\n{}\n{}",
            datetime, credential_scope, canonical_request_hash,
        );

        // 3. Derive Signing Key
        let k_date = hmac_sha256(format!("AWS4{}", self.secret_key).as_bytes(), &date);
        let k_region = hmac_sha256(&k_date, region);
        let k_service = hmac_sha256(&k_region, service);
        let k_signing = hmac_sha256(&k_service, "aws4_request");
        let signature = hex::encode(hmac_sha256(&k_signing, &string_to_sign));

        let authorization = format!(
            "SDK-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
            self.access_key, credential_scope, signed_headers, signature,
        );

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
            .header("X-Sdk-Date", &datetime)
            .header("Authorization", &authorization);

        if let Some(b) = body {
            req = req.json(&b);
        }

        let resp = req.send().await?;
        let status = resp.status();
        let result: serde_json::Value = resp.json().await?;

        if !status.is_success() {
            let msg = result["error_msg"]
                .as_str()
                .or_else(|| result["error"]["message"].as_str())
                .unwrap_or("unknown error");
            return Err(CertError::DnsProvider {
                provider: self.name(),
                detail: format!("[{}] {}", status, msg),
            });
        }

        Ok(result)
    }
}

#[async_trait]
impl DnsProviderOps for HuaweiProvider {
    fn name(&self) -> &'static str {
        "huawei"
    }

    async fn create_txt(&self, domain: &str, value: &str) -> Result<(), CertError> {
        let zone_id = self.get_zone_id(domain).await?;
        let (zone, sub) = Self::split_domain(domain);
        let name = if sub.is_empty() {
            format!("_acme-challenge.{}.", zone)
        } else {
            format!("_acme-challenge.{}.{}.", sub, zone)
        };

        let body = serde_json::json!({
            "name": name,
            "type": "TXT",
            "ttl": 60,
            "records": [format!("\"{}\"", value)],
        });

        self.call_api(
            "POST",
            &format!("/v2/zones/{}/recordsets", zone_id),
            Some(body),
        )
        .await?;
        Ok(())
    }

    async fn delete_txt(&self, domain: &str, _value: &str) -> Result<(), CertError> {
        let zone_id = self.get_zone_id(domain).await?;
        let (zone, sub) = Self::split_domain(domain);
        let name = if sub.is_empty() {
            format!("_acme-challenge.{}.", zone)
        } else {
            format!("_acme-challenge.{}.{}.", sub, zone)
        };

        if let Some(rs_id) = self.get_recordset_id(&zone_id, &name).await? {
            self.call_api(
                "DELETE",
                &format!("/v2/zones/{}/recordsets/{}", zone_id, rs_id),
                None,
            )
            .await?;
        }
        Ok(())
    }
}

// -- 签名辅助函数 --

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
