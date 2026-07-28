use crate::server::db::models::tls_cert::TlsCert;
use rocket::serde::{Deserialize, Serialize};

/// 证书列表响应（不含私钥）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertListRes {
    pub id: Option<i64>,
    pub domain: Option<String>,
    pub issuer: Option<String>,
    pub issued_at: Option<String>,
    pub expires_at: Option<String>,
    pub status: Option<String>,
    pub auto_renew: Option<bool>,
    pub create_time: Option<String>,
    pub remark: Option<String>,
}

impl From<TlsCert> for CertListRes {
    fn from(c: TlsCert) -> Self {
        let status = Some(format!("{:?}", c.effective_status()));
        CertListRes {
            id: c.id,
            domain: c.domain,
            issuer: c.issuer,
            issued_at: c.issued_at.map(|dt| dt.format("YYYY-MM-DD hh:mm:ss")),
            expires_at: c.expires_at.map(|dt| dt.format("YYYY-MM-DD hh:mm:ss")),
            status,
            auto_renew: c.auto_renew,
            create_time: c.create_time.map(|dt| dt.format("YYYY-MM-DD hh:mm:ss")),
            remark: c.remark,
        }
    }
}

/// 证书详情响应（含证书 PEM，不含私钥）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertDetailRes {
    pub id: Option<i64>,
    pub domain: Option<String>,
    pub cert_pem: Option<String>,
    pub issuer: Option<String>,
    pub issued_at: Option<String>,
    pub expires_at: Option<String>,
    pub status: Option<String>,
    pub auto_renew: Option<bool>,
    pub create_time: Option<String>,
    pub remark: Option<String>,
}

impl From<TlsCert> for CertDetailRes {
    fn from(c: TlsCert) -> Self {
        let status = Some(format!("{:?}", c.effective_status()));
        CertDetailRes {
            id: c.id,
            domain: c.domain,
            cert_pem: c.cert_pem,
            issuer: c.issuer,
            issued_at: c.issued_at.map(|dt| dt.format("YYYY-MM-DD hh:mm:ss")),
            expires_at: c.expires_at.map(|dt| dt.format("YYYY-MM-DD hh:mm:ss")),
            status,
            auto_renew: c.auto_renew,
            create_time: c.create_time.map(|dt| dt.format("YYYY-MM-DD hh:mm:ss")),
            remark: c.remark,
        }
    }
}

/// 证书私钥响应（含证书 PEM 和私钥 PEM）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertKeyRes {
    pub cert_pem: Option<String>,
    pub key_pem: Option<String>,
}

impl From<TlsCert> for CertKeyRes {
    fn from(c: TlsCert) -> Self {
        CertKeyRes {
            cert_pem: c.cert_pem,
            key_pem: c.key_pem,
        }
    }
}
