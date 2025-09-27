use rocket::serde::{Deserialize, Serialize};
use protocol::common::req::PageReq;
use protocol::impl_pagination;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyAddOrUpdateReq {
    pub name: String,
    pub principal: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyListReq {
    pub filter_text: Option<String>,
    page: PageReq
}
impl_pagination!(ApiKeyListReq);
