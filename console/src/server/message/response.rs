use crate::server::db::models::message::Message;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageCountRes {
    pub info: usize,
    pub warn: usize,
    pub error: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageListRes {
    #[serde(flatten)]
    pub inner: Message,
}
