mod client;
mod error;
mod response;
mod router;
mod sse;

use crate::context::{HCM, HttpContextWrapper};
use crate::openapi::client::HTTP_CLIENT;
use crate::openapi::error::GatewayError;
use crate::openapi::response::GatewayResponse;
use crate::openapi::sse::SseEvent;
use crate::router::ROUTER;
use protocol::gateway::RequestContext;
use reqwest::Url;
use rocket::async_stream::stream;
use rocket::http::hyper;
use rocket::http::uri::fmt::Path;
use rocket::request::FromSegments;
use rocket::serde::json::{Value, serde_json};
use rocket::yansi::Paint;
use rocket::{get, post, route};
use std::io;
use std::io::Bytes;
use std::path::PathBuf;
use std::pin::Pin;
use tokio_util::io::StreamReader;

/// OpenAPI统一入口
///
/// 要求：
/// - 所有请求都必须有响应，无论成功或失败
/// - 接口内部不处理任何业务逻辑，需转发到具体服务上处理
/// - 同时支持流式和非流式响应
/// - 流式响应支持恢复（插件实现）
///
#[post("/<path..>")]
pub async fn call(wrapper: HttpContextWrapper, path: PathBuf) -> GatewayResponse {
    let context = &wrapper.0.request;

    let path = &format!("/{}", path.to_str().unwrap());

    // 通过path获取对应的路由配置(可以考虑放到Fairing做？)
    let route = match ROUTER.matches(path) {
        Some(r) => r,
        None => {
            // 没有匹配到路由，返回502错误
            log::warn!("No route matched for path: {}", path);
            return GatewayResponse::Error(GatewayError::BadGateway);
        }
    };

    log::info!("匹配到路由：{:?}", route);

    // 服务ID
    let service_id = route.service_id.clone();

    // 负载URL
    let mut url = match Url::parse(&format!("lb://{}{}", service_id, path)) {
        Ok(url) => url,
        // 理论上不会执行到这里
        Err(e) => {
            log::error!("parse load balance url error: {}", e);
            return GatewayResponse::Error(GatewayError::BadGateway);
        }
    };

    // 添加query参数，如果有的话
    if let Some(query) = context.query.get() {
        if query != "" {
            url.set_query(Some(query));
        }
    }

    let url = url.as_str();
    log::info!("负载地址：{} {}", context.method, url);

    // 转发请求
    let response = HTTP_CLIENT.get(url, context.headers.clone()).await;
    // 获取响应
    match response {
        Ok(response) => match response {
            Ok(response) => GatewayResponse::Json(response.text().await.unwrap().into()),
            // 服务本身错误，如无响应等
            Err(_) => GatewayResponse::Error(GatewayError::ServiceUnavailable),
        },
        // 网关内部错误，如无可用实例、构建url失败、内部异常等
        Err(e) => {
            log::error!("{}", e);
            GatewayResponse::Error(GatewayError::BadGateway)
        }
    }

    // 封装响应

    // 返回

    // let stream = stream! {
    //     for i in 0..10 {
    //         let data = SseEvent::Data(format!("hello world {}", i)).to_string().into_bytes();
    //         yield Ok::<_, io::Error>(io::Cursor::new(data));
    //         tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    //     }
    // };
    // let stream_reader = Box::new(StreamReader::new(Box::pin(stream)));

    // Json响应
    //GatewayResponse::Json(context.get_path().into())

    // SSE响应
    //GatewayResponse::SSE(stream_reader)
}
