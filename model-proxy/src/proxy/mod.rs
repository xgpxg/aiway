pub mod api;
#[allow(clippy::module_inception)]
mod proxy;
mod request;
mod response;

use dashmap::DashMap;
use protocol::SV;
use protocol::gateway::{HttpContext, RequestContext, ResponseContext};
pub use proxy::Proxy;
pub use response::ModelError;
use rocket::data::ToByteUnit;
use rocket::request::{FromRequest, Outcome};
use std::sync::Arc;

/// 请求上下文，从gateway处复制的
pub struct HttpContextWrapper(pub Arc<HttpContext>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for HttpContextWrapper {
    type Error = &'r str;

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        // 注意Key为小写
        let headers = req
            .headers()
            .iter()
            // 移除不需要透传到下游服务的Header
            .filter(|h| h.name().ne("content-length") && h.name().ne("authorization"))
            .map(|h| (h.name().to_string(), h.value().to_string()))
            .collect::<DashMap<String, String>>();

        // 请求ID
        let request_id = uuid::Uuid::new_v4().to_string();
        // 请求时间戳
        let request_time = chrono::Local::now().timestamp_millis().to_string();

        // 请求上下文
        let request_context = RequestContext {
            request_id: request_id.to_string(),
            request_ts: request_time.parse().unwrap(),
            method: SV::new(req.method().as_str().into()),
            path: SV::new(req.uri().path().to_string()),
            headers,
            query: req
                .query_fields()
                .map(|q| (q.name.to_string(), q.value.to_string()))
                .collect::<DashMap<String, String>>(),
            body: Default::default(),
            state: Default::default(),
            route: SV::empty(),
            routing_url: SV::empty(),
            routing_path: SV::new(req.uri().path().to_string()),
            host: req.host().unwrap().to_string(),
        };

        // 响应上下文
        let response_context = ResponseContext::default();

        let context = HttpContext {
            request: request_context,
            response: response_context,
        };
        Outcome::Success(HttpContextWrapper(Arc::new(context)))
    }
}
