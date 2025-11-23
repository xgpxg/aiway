mod header;
mod manager;

use crate::extract_error;
pub use header::Headers;
pub use manager::HCM;
use protocol::gateway::HttpContext;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use std::sync::Arc;

/// 请求上下文包装器
///
/// - 在任意接口可通过该包装器取到请求上下文。
/// - HttpContext在请求中共享。
/// - RequestContext会在响应客户端前清理。
///
pub struct HttpContextWrapper(pub Arc<HttpContext>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for HttpContextWrapper {
    type Error = &'r str;

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        // 如果请求阶段的faring中设置了错误，则不会调用路由转发请求，在此返回错误
        if let Some(error) = extract_error!(req) {
            return Outcome::Error((Status::from_code(error.0).unwrap(), error.1));
        }
        let context = HCM.get_from_request(req);
        Outcome::Success(HttpContextWrapper(context))
    }
}
