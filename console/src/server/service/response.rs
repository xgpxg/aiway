use crate::server::db::models::service::Service;
use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceListRes {
    #[serde(flatten)]
    pub inner: Service,
}
