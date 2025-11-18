use protocol::common::req::PageReq;
use protocol::gateway::alert::AlertLevel;
use protocol::impl_pagination;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageListReq {
    pub page: PageReq,
    pub filter_text: Option<String>,
    pub level: Option<AlertLevel>,
}
impl_pagination!(MessageListReq);
