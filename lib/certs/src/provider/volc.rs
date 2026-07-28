use async_trait::async_trait;
use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::Client;
use sha2::{Digest, Sha256};

use crate::CertError;
use crate::provider::DnsProviderOps;

type HmacSha256 = Hmac<Sha256>;

/// 火山引擎 DNS API 实现（V4 HMAC-SHA256 签名）
pub struct VolcProvider {
    access_key: String,
    secret_key: String,
    client: Client,
}

impl VolcProvider {
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

    /// 获取 zone ID（ZoneName 不包含尾部点）
    async fn get_zone_id(&self, domain: &str) -> Result<String, CertError> {
        let parts: Vec<&str> = domain.split('.').collect();
        for i in 0..parts.len().saturating_sub(1) {
            let name = parts[i..].join(".");
            let resp = self
                .call_api(
                    "ListZones",
                    &serde_json::json!({
                        "Limit": 50,
                    }),
                )
                .await?;
            if let Some(zones) = resp["Result"]["Zones"].as_array() {
                for zone in zones {
                    if zone["ZoneName"]
                        .as_str()
                        .is_some_and(|z| z == name || z == format!("{}.", name))
                        && let Some(id) = zone["ZID"].as_i64().or_else(|| zone["ZoneId"].as_i64())
                    {
                        return Ok(id.to_string());
                    }
                }
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
                "ListRecordSets",
                &serde_json::json!({
                    "ZID": zone_id.parse::<i64>().unwrap_or(0),
                    "SearchThirdLevel": false,
                    "PageNumber": 1,
                    "PageSize": 50,
                }),
            )
            .await?;
        Ok(resp["Result"]["Records"]
            .as_array()
            .and_then(|records| {
                records.iter().find(|r| {
                    (r["Name"].as_str() == Some(name)) && r["Type"].as_str() == Some("TXT")
                })
            })
            .and_then(|r| {
                r["RecordSetID"]
                    .as_i64()
                    .or_else(|| r["RecordSetId"].as_i64())
            })
            .map(|id| id.to_string()))
    }

    /// 带 V4 签名的 API 调用
    async fn call_api(
        &self,
        action: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, CertError> {
        let host = "open.volcengineapi.com";
        let region = "cn-beijing";
        let service = "dns";
        let version = "2018-08-01";
        let now = Utc::now();
        let date = now.format("%Y%m%d").to_string();
        let datetime = now.format("%Y%m%dT%H%M%SZ").to_string();

        let payload = serde_json::to_string(body).map_err(|e| CertError::DnsProvider {
            provider: self.name(),
            detail: format!("Serialize error: {}", e),
        })?;
        let payload_hash = hex::encode(Sha256::digest(payload.as_bytes()));

        let query = format!("Action={}&Version={}", action, version);
        let signed_headers = "content-type;host;x-content-sha256;x-date";
        let canonical_headers = format!(
            "content-type:application/json\nhost:{}\nx-content-sha256:{}\nx-date:{}\n",
            host, payload_hash, datetime,
        );

        // 1. Canonical Request
        let canonical_request = format!(
            "POST\n/\n{}\n{}\n{}\n{}",
            query, canonical_headers, signed_headers, payload_hash,
        );
        let canonical_request_hash = hex::encode(Sha256::digest(canonical_request.as_bytes()));

        // 2. String to Sign
        let credential_scope = format!("{}/{}/{}/request", date, region, service);
        let string_to_sign = format!(
            "HMAC-SHA256\n{}\n{}\n{}",
            datetime, credential_scope, canonical_request_hash,
        );

        // 3. Derive Signing Key
        let k_date = hmac_sha256(
            format!("volcengine_secret{}", self.secret_key).as_bytes(),
            &date,
        );
        let k_region = hmac_sha256(&k_date, region);
        let k_service = hmac_sha256(&k_region, service);
        let k_signing = hmac_sha256(&k_service, "request");
        let signature = hex::encode(hmac_sha256(&k_signing, &string_to_sign));

        let authorization = format!(
            "HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
            self.access_key, credential_scope, signed_headers, signature,
        );

        let url = format!("https://{}?{}", host, query);
        let resp = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Host", host)
            .header("X-Content-Sha256", &payload_hash)
            .header("X-Date", &datetime)
            .header("Authorization", &authorization)
            .body(payload)
            .send()
            .await?;

        let result: serde_json::Value = resp.json().await?;

        if let Some(err) = result
            .get("Error")
            .or_else(|| result.get("ResponseMetadata").and_then(|m| m.get("Error")))
        {
            let code = err
                .get("Code")
                .and_then(|c| c.as_str())
                .unwrap_or("Unknown");
            let msg = err
                .get("Message")
                .and_then(|m| m.as_str())
                .unwrap_or("unknown error");
            return Err(CertError::DnsProvider {
                provider: self.name(),
                detail: format!("[{}] {}", code, msg),
            });
        }

        Ok(result)
    }
}

#[async_trait]
impl DnsProviderOps for VolcProvider {
    fn name(&self) -> &'static str {
        "volcengine"
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
            "ZID": zone_id.parse::<i64>().unwrap_or(0),
            "Name": name,
            "Type": "TXT",
            "TTL": 60,
            "Value": value,
        });

        self.call_api("CreateRecordSet", &body).await?;
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
            let body = serde_json::json!({
                "ZID": zone_id.parse::<i64>().unwrap_or(0),
                "RecordSetID": record_id.parse::<i64>().unwrap_or(0),
            });
            self.call_api("DeleteRecordSet", &body).await?;
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
