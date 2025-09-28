use protocol::common::req::PageReq;
use protocol::impl_pagination;
use rbatis::rbdc::DateTime;
use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyAddOrUpdateReq {
    pub name: String,
    pub principal: Option<String>,
    pub exp_time: Option<DateTime>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyListReq {
    pub filter_text: Option<String>,
    page: PageReq,
}
impl_pagination!(ApiKeyListReq);
