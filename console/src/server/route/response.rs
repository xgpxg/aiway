use crate::server::db::models::route::Route;
use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RouteListRes {
    #[serde(flatten)]
    pub inner: Route,
}
