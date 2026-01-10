//! # OpenAPI统一入口
//!
//! 要求：
//! - 所有请求都必须有响应，无论成功或失败
//! - 接口内部不处理任何业务逻辑，需转发到具体服务上处理
//! - 同时支持流式和非流式响应
//! - 流式响应支持恢复（插件实现）
//!
mod client;
mod error;
mod response;
#[deprecated]
#[allow(unused)]
mod sse;

use crate::openapi::client::HTTP_CLIENT;
use crate::openapi::error::GatewayError;
use crate::openapi::response::{GatewayResponse, ResponseExt};
use alert::Alert;
use context::HttpContextWrapper;
use reqwest::{StatusCode, Url};
use rocket::data::ToByteUnit;
use rocket::{Data, delete, get, head, options, patch, post, put};
use std::path::PathBuf;

async fn set_body(wrapper: &HttpContextWrapper, body: Data<'_>) -> Result<(), GatewayError> {
    wrapper.0.request.set_body(
        body.open(10.megabytes())
            .into_bytes()
            .await
            .map_err(|e| GatewayError::IO(e.to_string()))?
            .to_vec()
            .into(),
    );
    Ok(())
}

#[get("/<path..>")]
pub async fn call_get(wrapper: HttpContextWrapper, path: PathBuf) -> GatewayResponse {
    handle(wrapper, path).await
}

#[post("/<path..>", data = "<body>")]
pub async fn call_post(
    wrapper: HttpContextWrapper,
    path: PathBuf,
    body: Data<'_>,
) -> GatewayResponse {
    if let Err(e) = set_body(&wrapper, body).await {
        return GatewayResponse::Error(e);
    }
    handle(wrapper, path).await
}

#[put("/<path..>", data = "<body>")]
pub async fn call_put(
    wrapper: HttpContextWrapper,
    path: PathBuf,
    body: Data<'_>,
) -> GatewayResponse {
    if let Err(e) = set_body(&wrapper, body).await {
        return GatewayResponse::Error(e);
    }
    handle(wrapper, path).await
}

#[patch("/<path..>", data = "<body>")]
pub async fn call_patch(
    wrapper: HttpContextWrapper,
    path: PathBuf,
    body: Data<'_>,
) -> GatewayResponse {
    if let Err(e) = set_body(&wrapper, body).await {
        return GatewayResponse::Error(e);
    }
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

async fn handle(wrapper: HttpContextWrapper, _path: PathBuf) -> GatewayResponse {
    let request_context = &wrapper.0.request;

    // 实际路由路径
    let path = &request_context.get_path();

    // 路由的实际地址，该地址已经由负载均衡处理过，可能是IP或域名
    let routing_url = request_context.get_routing_url().unwrap();
    let mut url = match Url::parse(&format!(
        "{}/{}",
        routing_url.trim_end_matches('/'),
        path.trim_start_matches("/")
    )) {
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
        for q in request_context.query.iter() {
            query_pairs.append_pair(q.key(), q.value());
        }
    }

    // 请求头
    let headers = request_context.headers.clone();

    //log::info!("最终请求地址：{} {}", context.method, url);

    // 请求方法
    let method = request_context.get_method().unwrap_or_default();

    // 这里clone可能有性能问题
    let body = request_context.get_body().cloned();

    // 转发请求
    let response = HTTP_CLIENT
        .request(method, url, headers, body.unwrap_or_default())
        .await;

    let response_context = &wrapper.0.response;
    // 获取响应
    match response {
        Ok(response) => match response {
            // 返回响应（服务本身返回异常，如4xx、5xx时也会走这里）
            Ok(response) => {
                response.into_context(response_context).await;
                GatewayResponse::Success
            }
            // 请求服务时错误，如无响应等
            Err(e) => {
                log::error!("call service error: {:?}", e);
                response_context.set_status(
                    e.status()
                        .unwrap_or(StatusCode::SERVICE_UNAVAILABLE)
                        .as_u16(),
                );
                GatewayResponse::Error(GatewayError::ServiceUnavailable)
            }
        },
        // 网关内部错误，如无可用实例、构建url失败、内部异常等
        Err(e) => {
            log::error!("gateway inner error: {}", e);
            response_context.set_status(502);
            Alert::error("Gateway Inner Error", &e.to_string());
            GatewayResponse::Error(GatewayError::BadGateway)
        }
    }
}
