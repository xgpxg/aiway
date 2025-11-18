use crate::args::Args;
use crate::server::auth::UserPrincipal;
use crate::server::_template::request::DemoReq;
use crate::server::_template::response::DemoRes;
use protocol::common::res::{PageRes, Res};
use protocol::logg::LogEntry;
use rocket::serde::json::Json;
use rocket::{State, post, routes};

pub fn routes() -> Vec<rocket::Route> {
    routes![list]
}

#[post("/list", data = "<req>")]
pub async fn list(req: Json<DemoReq>, _user: UserPrincipal) -> Res<PageRes<DemoRes>> {
    todo!()
}
