use busi::impl_pagination;
use busi::req::PageReq;
use crate::server::db::models::domain::DomainStatus;
use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainAddOrUpdateReq {
    pub id: Option<i64>,
    pub domain: String,
    pub protocol: String,
    pub cert: Option<String>,
    pub key: Option<String>,
    pub remark: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainListReq {
    page: PageReq,
    pub filter_text: Option<String>,
    pub status: Option<String>,
    pub protocol: Option<String>,
}
impl_pagination!(DomainListReq);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStatusReq {
    pub id: i64,
    pub status: DomainStatus,
}
