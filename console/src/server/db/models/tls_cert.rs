use crate::server::cert::CertListReq;
use derive_builder::Builder;
use rbatis::rbdc::DateTime;
use rbatis::{crud, htmlsql_select_page};
use rocket::serde::{Deserialize, Serialize};

/// TLS 证书
#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
#[builder(default)]
pub struct TlsCert {
    pub id: Option<i64>,
    /// 主域名，如 example.com 或 *.example.com
    pub domain: Option<String>,
    /// PEM 格式证书内容（含证书链）
    pub cert_pem: Option<String>,
    /// PEM 格式私钥内容
    pub key_pem: Option<String>,
    /// 签发机构
    pub issuer: Option<String>,
    /// 签发时间
    #[serde(serialize_with = "crate::server::common::serialize_datetime")]
    pub issued_at: Option<DateTime>,
    /// 过期时间
    #[serde(serialize_with = "crate::server::common::serialize_datetime")]
    pub expires_at: Option<DateTime>,

    /// 是否自动续期
    #[serde(deserialize_with = "crate::server::common::deserialize_bool_from_int")]
    pub auto_renew: Option<bool>,
    /// 创建人ID
    pub create_user_id: Option<i64>,
    /// 创建时间
    #[serde(serialize_with = "crate::server::common::serialize_datetime")]
    pub create_time: Option<DateTime>,
    /// 更新时间
    #[serde(serialize_with = "crate::server::common::serialize_datetime")]
    pub update_time: Option<DateTime>,
    /// 备注
    pub remark: Option<String>,
    /// 是否删除
    pub is_delete: Option<i8>,
}

/// 证书状态
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum CertStatus {
    /// 有效
    #[default]
    Active,
    /// 已过期
    Expired,
}

crud!(TlsCert {});
impl TlsCert {
    /// 根据 expires_at 动态判断实际状态
    pub fn effective_status(&self) -> CertStatus {
        match &self.expires_at {
            Some(dt) if dt.unix_timestamp() * 1000 < chrono::Utc::now().timestamp_millis() => {
                CertStatus::Expired
            }
            _ => CertStatus::Active,
        }
    }
}

htmlsql_select_page!(list_page(param: &CertListReq) -> TlsCert => "src/server/db/mapper/tls_cert.html");
