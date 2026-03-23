use crate::Args;
use aiway_protocol::context::HttpContext;
use aiway_protocol::model::Provider;
use bytes::Bytes;
use pingora::{Error, ErrorType};
use pingora::prelude::Session;
use reqwest::Response;
use serde_json::Value;
use tokio_stream::StreamExt;

mod components;
mod proxy;

use crate::handler::plugin::PluginType;
use crate::handler::{HttpError, plugin};
use crate::model_proxy::proxy::Proxy;
pub use components::ModelFactory;

pub async fn init(_: &Args) {
    ModelFactory::init().await;
}

/// 处理模型请求
pub(crate) async fn handle_model_request(
    session: &mut Session,
    path: &str,
    ctx: &mut HttpContext,
) -> pingora::Result<bool, Box<Error>> {
    // 读取请求体（已消费，后续不可重复读取）
    let mut body = session.read_request_body().await?.unwrap_or_default();
    log::info!("[ModelProxy] 开始处理模型请求，路径：{}", path);

    // 解析模型和提供商信息
    let (model, provider) =
        parse_model_info(&mut body).map_err(|e| Error::new(ErrorType::HTTPStatus(e.0)))?;

    log::info!(
        "[ModelProxy] 解析到模型：{}, 提供商：{}",
        model,
        provider.name
    );

    ctx.insert_state(HttpContext::MODEL_PROXY_MODEL, model.to_string());
    ctx.insert_state(HttpContext::MODEL_PROXY_PROVIDER, provider.clone());

    // 插件类型
    let plugin_type = PluginType::Model {
        model_name: model.to_string(),
        provider_name: provider.name.clone(),
    };

    // 执行请求阶段插件
    log::info!("[ModelProxy] 执行请求阶段插件");
    if let Err(e) = execute_request_plugins(session, &plugin_type, ctx).await {
        log::error!("[ModelProxy] 请求阶段插件执行失败：{:?}", e);
        return crate::service::send_error_response(session, e.0, e.1.into())
            .await
            .map(|_| true);
    }

    // 执行请求体阶段插件
    let mut body_opt = Some(body);
    log::info!("[ModelProxy] 执行请求体阶段插件");
    if let Err(e) = execute_request_body_plugins(plugin_type.clone(), &mut body_opt, ctx).await {
        log::error!("[ModelProxy] 请求体阶段插件执行失败：{:?}", e);
        return crate::service::send_error_response(session, e.0, e.1.into())
            .await
            .map(|_| true);
    }

    // 检查请求体是否存在
    let body = match body_opt {
        Some(b) => b,
        None => {
            log::error!("[ModelProxy] 请求体为空");
            return crate::service::send_error_response(
                session,
                500,
                "Request body is none".into(),
            )
            .await
            .map(|_| true);
        }
    };

    // 调用模型 API
    log::info!("[ModelProxy] 调用模型 API");
    let response = match Proxy::call(Some(body), ctx).await {
        Ok(resp) => resp,
        Err(e) => {
            log::error!("[ModelProxy] 调用模型 API 失败：{:?}", e);
            let (status, message) = e.into_status_message();
            return crate::service::send_error_response(session, status, message)
                .await
                .map(|_| true);
        }
    };

    // 执行响应阶段插件并发送响应
    log::info!("[ModelProxy] 执行响应阶段插件并发送响应");
    execute_response_and_send(session, plugin_type, response, ctx).await
}

/// 解析模型信息
fn parse_model_info(body: &mut Bytes) -> Result<(String, Provider), HttpError> {
    // 转为JSON，提取模型名称
    let mut body_json =
        serde_json::from_slice::<Value>(body).map_err(|e| HttpError::new(400, &e.to_string()))?;
    let model = body_json["model"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| HttpError::new(400, "Missing 'model' field"))?;

    // 从模型的可用提供商中获取一个
    let provider =
        ModelFactory::get_provider(&model).map_err(|e| HttpError::new(400, &e.to_string()))?;

    log::info!(
        "[ModelProxy] 解析模型信息 - 原始模型：{}, 提供商：{}",
        model,
        provider.name
    );

    // 模型名称映射
    if let Some(target_model_name) = &provider.target_model_name
        && !target_model_name.is_empty()
    {
        log::info!(
            "[ModelProxy] 替换模型名称：{} -> {}",
            model,
            target_model_name
        );
        body_json["model"] = Value::String(provider.target_model_name.clone().unwrap());
    }

    // 修改请求体
    *body = serde_json::to_vec(&body_json).unwrap().into();

    Ok((model, provider))
}

/// 执行请求阶段插件
#[inline]
async fn execute_request_plugins(
    session: &mut Session,
    plugin_type: &PluginType,
    ctx: &mut HttpContext,
) -> Result<(), HttpError> {
    plugin::run_on_request(plugin_type.clone(), session.req_header_mut(), ctx).await
}

/// 执行请求体阶段插件
#[inline]
async fn execute_request_body_plugins(
    plugin_type: PluginType,
    body: &mut Option<Bytes>,
    ctx: &mut HttpContext,
) -> Result<(), HttpError> {
    plugin::run_on_request_body(plugin_type, body, ctx).await
}

/// 执行响应阶段插件并发送响应
async fn execute_response_and_send(
    session: &mut Session,
    plugin_type: PluginType,
    response: Response,
    ctx: &mut HttpContext,
) -> pingora::Result<bool, Box<Error>> {
    log::info!("[ModelProxy] 构建响应头，状态码：{}", response.status());

    // 构建响应头
    let mut response_header = crate::service::build_response_header(&response);

    // 执行响应阶段插件
    log::info!("[ModelProxy] 执行响应阶段插件");
    if let Err(e) = plugin::run_on_response(plugin_type.clone(), &mut response_header, ctx).await {
        log::error!("[ModelProxy] 响应阶段插件执行失败：{:?}", e);
        return crate::service::send_error_response(session, e.0, e.1.into())
            .await
            .map(|_| true);
    }

    // 发送响应头
    session
        .write_response_header(Box::new(response_header), false)
        .await?;

    // 发送响应体
    stream_response_body(session, response, plugin_type.clone(), ctx).await
}

/// 流式传输响应体
async fn stream_response_body(
    session: &mut Session,
    response: Response,
    plugin_type: PluginType,
    ctx: &mut HttpContext,
) -> pingora::Result<bool, Box<Error>> {
    let mut stream = response.bytes_stream();
    log::info!("[ModelProxy] 开始流式传输响应体");

    while let Some(item) = stream.next().await {
        let mut body = Some(item.unwrap());

        // 执行响应体阶段插件
        if let Err(e) = plugin::run_on_response_body(plugin_type.clone(), &mut body, ctx) {
            log::error!("[ModelProxy] 响应体阶段插件执行失败：{:?}", e);
            return crate::service::send_error_response(session, e.0, e.1.into())
                .await
                .map(|_| true);
        }

        session.write_response_body(body, false).await?;
    }

    // 发送结束标记
    log::info!("[ModelProxy] 流式传输完成");
    session.write_response_body(None, true).await?;
    Ok(true)
}
