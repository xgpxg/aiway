use crate::server::plugin::request::PluginAddOrUpdateReq;
use protocol::common::res::Res;
use rocket::form::Form;
use rocket::{post, routes};

pub fn routes() -> Vec<rocket::Route> {
    routes![add]
}

/// 新增插件
#[post("/add", data = "<req>")]
async fn add(req: Form<PluginAddOrUpdateReq<'_>>) -> Res<()> {
    todo!()
}
