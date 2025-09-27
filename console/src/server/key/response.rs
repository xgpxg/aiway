use crate::server::db::models::api_key::ApiKey;
use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyListRes {
    #[serde(flatten)]
    pub inner: ApiKey,
}
