use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialOrd, PartialEq)]
pub struct ApiKeySync {
    pub secret: String,
    pub action: Action,
    pub exp_time: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, PartialOrd, PartialEq)]
pub enum Action {
    Create,
    Update,
    Delete,
}
