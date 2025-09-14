mod response;
mod router;

use crate::context::{RCM, RequestContextWrapper};
use protocol::gateway::RequestContext;
use rocket::http::uri::fmt::Path;
use rocket::request::FromSegments;
use rocket::{get, post};
use std::path::PathBuf;

#[post("/<path..>")]
pub async fn call(wrapper: RequestContextWrapper, path: PathBuf) {
    let context = wrapper.0;
    //println!("path: {:?}", path);
    //println!("{:?}", context);

    println!("{}", context.get_query());
    return;
}
