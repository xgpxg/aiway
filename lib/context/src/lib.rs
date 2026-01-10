//! # HTTP上下文
//! 适用于Rocket的请求和响应上下文处理
//!
mod header;
pub mod macros;
mod manager;

use dashmap::DashMap;
pub use header::Headers;
pub use manager::HCM;
use protocol::SV;
use protocol::gateway::{HttpContext, RequestContext, ResponseContext};
use rocket::fairing::Fairing;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::{Data, Request};
use std::sync::Arc;

pub struct HttpContextFairing;
impl HttpContextFairing {
    pub fn new() -> Self {
        HttpContextFairing {}
    }
}

impl Default for HttpContextFairing {
    fn default() -> Self {
        HttpContextFairing::new()
    }
}
#[rocket::async_trait]
impl Fairing for HttpContextFairing {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "HttpContextFairing",
            kind: rocket::fairing::Kind::Request,
        }
    }

    async fn on_request(&self, req: &mut Request<'_>, _data: &mut Data<'_>) {
        skip_if_error!(req);
        let context = HttpContextOnce::from_request(req).0;
        let request_id = context.request.request_id.clone();

        // 设置请求上下文
        // 注意需要在响应前清理该上下文，否则会导致内存泄漏
        HCM.set(request_id, Arc::new(context));
    }
}

/// 请求上下文包装器
///
/// - 注意需要调用[HttpContextFairing]，将上下文保存到HCM。
/// - 在任意接口可通过该包装器取到请求上下文。
/// - HttpContext在请求中共享。
/// - RequestContext会在响应客户端前清理。
///
/// 目前仅在gateway中使用。
///
pub struct HttpContextWrapper(pub Arc<HttpContext>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for HttpContextWrapper {
    type Error = &'r str;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // 如果请求阶段的faring中设置了错误，则不会调用路由转发请求，在此返回错误
        if let Some(error) = extract_error!(req) {
            return Outcome::Error((Status::from_code(error.0).unwrap(), error.1));
        }
        let context = HCM.get_from_request(req);
        Outcome::Success(HttpContextWrapper(context))
    }
}

/// 一次性的请求上下文包装器，不会被HCM管理。
pub struct HttpContextOnce(pub HttpContext);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for HttpContextOnce {
    type Error = &'r str;
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        Outcome::Success(HttpContextOnce::from_request(req))
    }
}

impl HttpContextOnce {
    pub fn from_request(req: &Request<'_>) -> Self {
        let headers = req
            .headers()
            .iter()
            // 移除不需要透传到下游服务的Header
            .filter(|h| h.name().ne("content-length") && h.name().ne("authorization"))
            .map(|h| (h.name().to_string(), h.value().to_string()))
            .collect::<DashMap<String, String>>();

        // 请求ID
        let request_id = req
            .headers()
            .get_one(Headers::REQUEST_ID)
            .map(|s| s.to_string())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        // 请求时间戳
        let request_time = req
            .headers()
            .get_one(Headers::REQUEST_TIME)
            .map(|s| s.to_string())
            .unwrap_or_else(|| chrono::Local::now().timestamp_millis().to_string());

        // 请求上下文
        let request_context = RequestContext {
            request_id: request_id.clone(),
            request_ts: request_time.parse().unwrap(),
            method: req.method().as_str().into(),
            path: req.uri().path().to_string().into(),
            headers,
            query: req
                .query_fields()
                .map(|q| (q.name.to_string(), q.value.to_string()))
                .collect::<DashMap<String, String>>(),
            body: Default::default(),
            state: Default::default(),
            route: SV::empty(),
            routing_url: SV::empty(),
            //routing_path: SV::new(req.uri().path().to_string()),
            host: req.host().unwrap().to_string(),
        };

        // 响应上下文
        let response_context = ResponseContext::default();

        let context = HttpContext {
            request: request_context,
            response: response_context,
        };

        HttpContextOnce(context)
    }
}
