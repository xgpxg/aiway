use chrono::NaiveDateTime;
use protocol::common::req::PageReq;
use rocket::serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct DemoReq {}
