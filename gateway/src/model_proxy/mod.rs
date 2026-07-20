use crate::Args;
use aiway_protocol::context::HttpContext;
use aiway_protocol::gateway::ModelCallLog;
use aiway_protocol::model::{Provider, TokenUsageConfig};
use bytes::Bytes;
use pingora::prelude::Session;
use pingora::{Error, ErrorType};
use reqwest::Response;
use serde_json::Value;
use tokio_stream::StreamExt;

mod components;
mod proxy;

use crate::handler::plugin::PluginType;
use crate::handler::{HandlerError, plugin};
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
    args: &Args,
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
        return crate::service::send_error_response(session, e.0, e.1)
            .await
            .map(|_| true);
    }

    // 执行请求体阶段插件
    let mut body_opt = Some(body);
    log::info!("[ModelProxy] 执行请求体阶段插件");
    if let Err(e) = execute_request_body_plugins(plugin_type.clone(), &mut body_opt, ctx).await {
        log::error!("[ModelProxy] 请求体阶段插件执行失败：{:?}", e);
        return crate::service::send_error_response(session, e.0, e.1)
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
    let result = execute_response_and_send(session, plugin_type, response, ctx).await;

    // 记录模型调用日志
    // 模型调用日志为单独的日志索引，不和通用的请求日志混在一起
    log_model_call(ctx, &format!("{}:{}", args.address, args.port));

    result
}

/// 解析模型信息
fn parse_model_info(body: &mut Bytes) -> Result<(String, Provider), HandlerError> {
    // 转为JSON，提取模型名称
    let mut body_json = serde_json::from_slice::<Value>(body)
        .map_err(|e| HandlerError::new(400, &e.to_string()))?;
    let model = body_json["model"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| HandlerError::new(400, "Missing 'model' field"))?;

    // 从模型的可用提供商中获取一个
    let provider =
        ModelFactory::get_provider(&model).map_err(|e| HandlerError::new(400, &e.to_string()))?;

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
) -> Result<(), HandlerError> {
    plugin::run_on_request(plugin_type.clone(), session.req_header_mut(), ctx).await
}

/// 执行请求体阶段插件
#[inline]
async fn execute_request_body_plugins(
    plugin_type: PluginType,
    body: &mut Option<Bytes>,
    ctx: &mut HttpContext,
) -> Result<(), HandlerError> {
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

    // 存储响应状态码
    ctx.insert_state(
        HttpContext::RESPONSE_STATUS_CODE,
        response.status().as_u16(),
    );

    // 构建响应头
    let mut response_header = crate::service::build_response_header(&response);

    // 执行响应阶段插件
    log::info!("[ModelProxy] 执行响应阶段插件");
    if let Err(e) = plugin::run_on_response(plugin_type.clone(), &mut response_header, ctx).await {
        log::error!("[ModelProxy] 响应阶段插件执行失败：{:?}", e);
        return crate::service::send_error_response(session, e.0, e.1)
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

/// 流式传输响应体，同时提取 Token 用量和 TTFT
async fn stream_response_body(
    session: &mut Session,
    response: Response,
    plugin_type: PluginType,
    ctx: &mut HttpContext,
) -> pingora::Result<bool, Box<Error>> {
    // 判断是否 SSE 流式响应
    let is_sse = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.contains("text/event-stream"))
        .unwrap_or(false);

    // 标记 SSE 状态到上下文
    if is_sse {
        ctx.insert_state(HttpContext::IS_SSE, true);
    }

    let mut stream = response.bytes_stream();
    let mut first_chunk_ts: Option<i64> = None;

    // 非流式：收集完整 body；流式：保留最后一个完整事件块
    let mut full_body: Option<Vec<u8>> = if !is_sse { Some(Vec::new()) } else { None };
    let mut last_event: Option<Vec<u8>> = if is_sse { Some(Vec::new()) } else { None };

    while let Some(item) = stream.next().await {
        // 记录首 chunk 到达时间
        if first_chunk_ts.is_none() {
            first_chunk_ts = Some(chrono::Local::now().timestamp_millis());
        }

        let mut body = Some(item.unwrap());
        let chunk = body.as_ref().unwrap();

        // 累积 body 数据
        if let Some(ref mut buf) = full_body {
            buf.extend_from_slice(chunk);
        }
        if let Some(ref mut buf) = last_event {
            // SSE 模式：累积所有事件，由 extract_and_store_tokens 提取最后一条有效 data 行
            buf.extend_from_slice(chunk);
        }

        // 执行响应体阶段插件
        if let Err(e) = plugin::run_on_response_body(plugin_type.clone(), &mut body, ctx) {
            log::error!("[ModelProxy] 响应体阶段插件执行失败：{:?}", e);
            return crate::service::send_error_response(session, e.0, e.1)
                .await
                .map(|_| true);
        }

        session.write_response_body(body, false).await?;
    }

    // Token 提取
    let provider = ctx.get_proxy_model_provider();
    if let Some(ref provider) = provider {
        if let Some(ref config) = provider.token_usage_config {
            let body_bytes = full_body.as_ref().or_else(|| last_event.as_ref());
            if let Some(bytes) = body_bytes {
                extract_and_store_tokens(bytes, config, ctx);
            }
        }
    }

    // TTFT 存入上下文
    if let Some(ts) = first_chunk_ts {
        ctx.insert_state(HttpContext::MODEL_TTFT_MS, ts - ctx.request_ts());
    }

    // 发送结束标记
    session.write_response_body(None, true).await?;
    Ok(true)
}

/// 按配置的 JSON 路径提取 Token 用量，并存入 HttpContext
fn extract_and_store_tokens(body: &[u8], config: &TokenUsageConfig, ctx: &HttpContext) {
    let json = match serde_json::from_slice::<Value>(body) {
        Ok(v) if v.is_object() => v,
        _ => {
            // SSE 格式：从最后一条 data: 行中提取 JSON
            let text = String::from_utf8_lossy(body);
            match text
                .lines()
                .filter_map(|line| line.strip_prefix("data: "))
                .filter(|s| !s.starts_with("[DONE]"))
                .last()
                .and_then(|s| serde_json::from_str::<Value>(s).ok())
            {
                Some(v) => v,
                None => return,
            }
        }
    };

    if let Some(path) = &config.prompt_tokens {
        if let Some(v) = json_path_extract(&json, path) {
            ctx.insert_state(HttpContext::MODEL_USAGE_PROMPT_TOKENS, v);
        }
    }
    if let Some(path) = &config.completion_tokens {
        if let Some(v) = json_path_extract(&json, path) {
            ctx.insert_state(HttpContext::MODEL_USAGE_COMPLETION_TOKENS, v);
        }
    }
    if let Some(path) = &config.total_tokens {
        if let Some(v) = json_path_extract(&json, path) {
            ctx.insert_state(HttpContext::MODEL_USAGE_TOTAL_TOKENS, v);
        }
    }
}

/// 按点号分隔路径从 JSON 中取值，如 "usage.prompt_tokens"
fn json_path_extract(json: &Value, path: &str) -> Option<i64> {
    let mut current = json;
    for part in path.split('.') {
        current = current.get(part)?;
    }
    current.as_i64()
}

/// 在响应发送完毕后记录模型调用日志
pub(crate) fn log_model_call(ctx: &HttpContext, node_address: &str) {
    let model_name = match ctx.get_proxy_model_name() {
        Some(name) => name,
        None => return,
    };
    let provider = match ctx.get_proxy_model_provider() {
        Some(p) => p,
        None => return,
    };

    let response_time = chrono::Local::now().timestamp_millis();
    let request_time = ctx.request_ts();
    let is_stream = ctx.is_sse();

    let ttft_ms: Option<i64> = ctx.get_state(HttpContext::MODEL_TTFT_MS);
    let prompt_tokens: Option<i64> = ctx.get_state(HttpContext::MODEL_USAGE_PROMPT_TOKENS);
    let completion_tokens: Option<i64> = ctx.get_state(HttpContext::MODEL_USAGE_COMPLETION_TOKENS);
    let total_tokens: Option<i64> = ctx.get_state(HttpContext::MODEL_USAGE_TOTAL_TOKENS);
    let status_code: Option<u16> = ctx.get_state(HttpContext::RESPONSE_STATUS_CODE);

    let log = ModelCallLog {
        request_id: ctx.request_id(),
        model_name,
        provider_name: provider.name,
        request_time,
        response_time,
        elapsed: response_time - request_time,
        ttft_ms: if is_stream { ttft_ms } else { None },
        status_code: status_code.unwrap_or(200),
        is_stream,
        prompt_tokens,
        completion_tokens,
        total_tokens,
        node_address: node_address.to_string(),
    };

    if let Ok(json) = serde_json::to_vec(&log) {
        logging::log_model_call(json);
    }
}
