use crate::server::db::models::model::Model;
use crate::server::db::models::model_provider::ModelProvider;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ModelListRes {
    #[serde(flatten)]
    pub inner: Model,
    pub providers: Vec<ModelProvider>,
}
