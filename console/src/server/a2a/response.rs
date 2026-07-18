use crate::server::db::models::agent::Agent;
use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentListRes {
    #[serde(flatten)]
    pub inner: Agent,
}
