mod header;
mod manager;

pub use header::Headers;
pub use manager::RCM;
use protocol::gateway::RequestContext;
use rocket::request::FromRequest;
use std::sync::Arc;

/// 请求上下文包装器
///
/// 在任意接口可通过该包装器取到请求上下文。
/// RequestContext会在响应客户端前清理
pub struct RequestContextWrapper(pub Arc<RequestContext>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RequestContextWrapper {
    type Error = ();

    async fn from_request(
        req: &'r rocket::Request<'_>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        let context = RCM.get_from_request(&req);
        rocket::request::Outcome::Success(RequestContextWrapper(context))
    }
}
