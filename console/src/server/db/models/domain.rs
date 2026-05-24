use crate::server::domain::DomainListReq;
use derive_builder::Builder;
use rbatis::rbdc::DateTime;
use rbatis::{crud, htmlsql_select_page};
use rocket::serde::{Deserialize, Serialize};

/// 域名
#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
#[builder(default)]
pub struct Domain {
    pub id: Option<i64>,
    /// 绑定域名
    pub domain: Option<String>,
    /// 协议：HTTP | HTTPS
    pub protocol: Option<Protocol>,
    /// PEM 格式证书内容（HTTPS 必填）
    pub cert: Option<String>,
    /// PEM 格式私钥内容（HTTPS 必填）
    pub cert_key: Option<String>,
    /// 状态：Disable | Ok
    pub status: Option<DomainStatus>,
    /// 创建人ID
    pub create_user_id: Option<i64>,
    /// 修改人ID
    pub update_user_id: Option<i64>,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum DomainStatus {
    /// 禁用
    #[default]
    Disable,
    /// 启用
    Ok,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum Protocol {
    #[default]
    HTTP,
    HTTPS,
}

crud!(Domain {});
htmlsql_select_page!(list_page(param: &DomainListReq) -> Domain => "src/server/db/mapper/domain.html");
