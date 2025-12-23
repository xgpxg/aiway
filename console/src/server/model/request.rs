use crate::server::db::models::model::{LbStrategy, ModelStatus};
use crate::server::db::models::model_provider::ModelProviderStatus;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ModelLisReq {}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelAddReq {
    pub name: String,
}
#[derive(Debug, Clone, Deserialize)]
pub struct ModelUpdateReq {
    pub id: i64,
    pub name: Option<String>,
    pub status: Option<ModelStatus>,
    pub lb_strategy: Option<LbStrategy>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct ProviderAddReq {
    pub model_id: i64,
    pub name: String,
    pub api_url: String,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProviderUpdateReq {
    pub id: i64,
    pub name: Option<String>,
    pub api_url: Option<String>,
    pub api_key: Option<String>,
    pub status: Option<ModelProviderStatus>,
}
