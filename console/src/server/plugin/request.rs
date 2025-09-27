use rocket::FromForm;
use rocket::fs::TempFile;
use rocket::serde::{Deserialize, Serialize};

#[derive(Debug,FromForm)]
pub struct PluginAddOrUpdateReq<'a> {
    pub name: String,
    pub version: String,
    pub file: TempFile<'a>,
}
