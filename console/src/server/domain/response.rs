use crate::server::db::models::domain::Domain;
use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainListRes {
    #[serde(flatten)]
    pub inner: Domain,
}
