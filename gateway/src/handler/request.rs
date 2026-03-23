use crate::handler::HttpResult;
use aiway_protocol::context::{HttpContext, SessionExt};
use pingora::prelude::*;
use aiway_protocol::common::header::Headers;

pub async fn request_handle(session: &mut Session, ctx: &mut HttpContext) -> HttpResult<()> {
    // let request_id = uuid::Uuid::new_v4().to_string();
    // let request_time = chrono::Local::now().timestamp_millis();
    //
    // session.set_request_header(Headers::REQUEST_ID, &request_id);
    // session.set_request_header(Headers::REQUEST_TIME, &request_time.to_string());

    Ok(())
}
