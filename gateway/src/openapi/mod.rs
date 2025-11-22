//! # OpenAPI统一入口
//!
//! 要求：
//! - 所有请求都必须有响应，无论成功或失败
//! - 接口内部不处理任何业务逻辑，需转发到具体服务上处理
//! - 同时支持流式和非流式响应
//! - 流式响应支持恢复（插件实现）
//!
mod client;
#[deprecated]
pub mod eep;
mod error;
mod response;
mod sse;

use crate::context::HttpContextWrapper;
use crate::openapi::client::HTTP_CLIENT;
use crate::openapi::error::GatewayError;
use crate::openapi::response::{GatewayResponse, ResponseExt};
use dashmap::DashMap;
use reqwest::Url;
use rocket::futures::StreamExt;
use rocket::{delete, get, head, options, patch, post, put};
use std::io;
use std::path::PathBuf;
use tokio_util::io::StreamReader;

#[get("/<path..>")]
pub async fn call_get(wrapper: HttpContextWrapper, path: PathBuf) -> GatewayResponse {
    handle(wrapper, path).await
}

#[post("/<path..>")]
pub async fn call_post(wrapper: HttpContextWrapper, path: PathBuf) -> GatewayResponse {
    handle(wrapper, path).await
}

#[put("/<path..>")]
pub async fn call_put(wrapper: HttpContextWrapper, path: PathBuf) -> GatewayResponse {
    handle(wrapper, path).await
}

#[patch("/<path..>")]
pub async fn call_patch(wrapper: HttpContextWrapper, path: PathBuf) -> GatewayResponse {
    handle(wrapper, path).await
}

#[delete("/<path..>")]
pub async fn call_delete(wrapper: HttpContextWrapper, path: PathBuf) -> GatewayResponse {
    handle(wrapper, path).await
}
#[head("/<path..>")]
pub async fn call_head(wrapper: HttpContextWrapper, path: PathBuf) -> GatewayResponse {
    handle(wrapper, path).await
}

#[options("/<path..>")]
pub async fn call_options(wrapper: HttpContextWrapper, path: PathBuf) -> GatewayResponse {
    handle(wrapper, path).await
}

async fn handle(wrapper: HttpContextWrapper, path: PathBuf) -> GatewayResponse {
    let context = &wrapper.0.request;
    // 获取匹配的路由
    // SAFE: 在routing fairing处理时已经验证，能走到这里来，一定会有值
    //let route = context.get_route().unwrap();
    //log::info!("匹配到路由：{:?}", route);

    // 原始请求路径
    //let path = path.to_string_lossy();
    //log::info!("原始请求路径：{?}", path);

    //实际路由路径
    let path = &context.routing_path;

    // 路由的实际地址，该地址已经有负载均衡处理过，可能是IP或域名
    let routing_url = context.get_routing_url().unwrap();
    let mut url = match Url::parse(&format!("{}/{}", routing_url, path)) {
        Ok(url) => url,
        // 理论上不会执行到这里
        Err(e) => {
            log::error!("parse load balance url error: {}", e);
            return GatewayResponse::Error(GatewayError::BadGateway);
        }
    };

    // 添加query参数，如果有的话
    {
        let mut query_pairs = url.query_pairs_mut();
        query_pairs.clear();
        for q in context.query.iter() {
            query_pairs.append_pair(q.key(), q.value());
        }
    }

    // 请求头
    let headers = context.headers.clone();

    //log::info!("最终请求地址：{} {}", context.method, url);

    // 请求方法
    let method = context.get_method().unwrap_or_default();

    // 这里clone可能有性能问题
    let body = context.get_body().cloned();

    // 转发请求
    let response = HTTP_CLIENT
        .request(method, url, headers, body.unwrap_or_default())
        .await;
    // 获取响应
    match response {
        Ok(response) => match response {
            // 返回响应
            Ok(response) => {
                // 透传状态码
                let status = response.status();
                // 处理SSE流
                if response.is_sse() {
                    let stream = response.bytes_stream();
                    let stream_reader =
                        StreamReader::new(stream.map(|result| {
                            result.map_err(|e| io::Error::other(e))
                        }));
                    return GatewayResponse::SSE(Box::new(stream_reader));
                }
                GatewayResponse::Raw(status.as_u16(), response.bytes().await.unwrap())
            }
            // 服务本身错误，如无响应等
            Err(e) => {
                log::error!("call service error: {:?}", e);
                GatewayResponse::Error(GatewayError::ServiceUnavailable)
            }
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
