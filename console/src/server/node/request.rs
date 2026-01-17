use busi::req::PageReq;
use busi::impl_pagination;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayNodeListReq {
    pub page: PageReq,
}
impl_pagination!(GatewayNodeListReq);
