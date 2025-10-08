use rocket::serde::{Deserialize, Serialize};
use crate::server::db::models::plugin::Plugin;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginListRes {
    #[serde(flatten)]
    pub inner: Plugin,
}
