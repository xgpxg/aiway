use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteAddReq {
    pub name: String,
    pub description: Option<String>,
    pub host: Option<String>,
    pub prefix: Option<String>,
    pub path: String,
    pub service: String,
    #[serde(default = "Default::default")]
    pub header: BTreeMap<String, String>,
    #[serde(default = "Default::default")]
    pub query: BTreeMap<String, String>,
    #[serde(default = "Default::default")]
    pub pre_filters: Vec<String>,
    #[serde(default = "Default::default")]
    pub post_filters: Vec<String>,
}
