use super::AcmeConfig;
use rocket::serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AcmeConfigUpdateReq {
    #[serde(flatten)]
    pub inner: AcmeConfig,
}
