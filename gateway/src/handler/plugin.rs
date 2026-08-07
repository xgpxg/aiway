use crate::components::GlobalPluginFactory;
use crate::handler::{HandlerError, HandlerResult};
use crate::model_proxy::ModelFactory;
use aiway_protocol::context::{
    HeaderOp, HttpContext, REQUEST_HEADER_PATCH, RESPONSE_HEADER_PATCH, REQUEST_URI_PATCH,
};
use bytes::Bytes;
use pingora::prelude::*;
use plugin_manager::{Outcome, PluginFactory, Response};

#[derive(Clone, Debug)]
pub enum PluginType {
    /// 全局插件
    Global,
    /// 路由插件
    Route,
    /// 模型插件
    Model {
        model_name: String,
        provider_name: String,
    },
}
/// 执行单个插件并处理错误（异步版本）
macro_rules! execute_plugin_async {
    ($plugin:expr, $ctx:expr, $method:ident $(, $arg:expr)*) => {{
        log::debug!("execute plugin: {}", $plugin.name);
        PluginFactory::$method(&$plugin, $ctx $(, $arg)*).await.map_err(|e| {
            // 插件自身异常，统一 502
            log::error!("execute plugin {} error: {}", $plugin.name, e);
            HandlerError::new(502, "BadPlugin")
        })?;
    }};
}

/// 执行单个插件并处理 Outcome（支持主动响应）
macro_rules! execute_plugin_outcome {
    ($plugin:expr, $ctx:expr, $method:ident) => {{
        log::debug!("execute plugin: {}", $plugin.name);
        PluginFactory::$method(&$plugin, $ctx).await.map_err(|e| {
            log::error!("execute plugin {} error: {}", $plugin.name, e);
            HandlerError::new(502, "BadPlugin")
        })?
    }};
}

/// 执行请求阶段插件
pub async fn run_on_request(
    plugin_type: PluginType,
    head: &mut RequestHeader,
    ctx: &mut HttpContext,
) -> HandlerResult<Option<Response>> {
    // 清除上一阶段的 HeaderOp / URI patch
    ctx.remove_any_state::<Vec<HeaderOp>>(REQUEST_HEADER_PATCH);
    ctx.remove_any_state::<http::Uri>(REQUEST_URI_PATCH);

    let mut respond: Option<Response> = None;
    match plugin_type {
        PluginType::Global => {
            let plugins = GlobalPluginFactory::get_plugins();
            for plugin in plugins.iter() {
                match execute_plugin_outcome!(plugin, ctx, on_request) {
                    Outcome::Continue => continue,
                    Outcome::Respond(resp) => { respond = Some(resp); break; }
                }
            }
        }
        PluginType::Route => {
            // SAFE：在 routing 时已经设置
            let route = ctx.get_route().unwrap();
            let plugins = &route.plugins;
            for plugin in plugins.iter() {
                match execute_plugin_outcome!(plugin, ctx, on_request) {
                    Outcome::Continue => continue,
                    Outcome::Respond(resp) => { respond = Some(resp); break; }
                }
            }
        }
        PluginType::Model {
            model_name,
            provider_name,
        } => {
            let provider = ModelFactory::get_special_provider(&model_name, &provider_name).unwrap();
            if let Some(plugin) = provider.plugins {
                log::info!(
                    "execute model provider request converter plugin: {}",
                    plugin.name
                );
                match execute_plugin_outcome!(plugin, ctx, on_request) {
                    Outcome::Continue => {}
                    Outcome::Respond(resp) => { respond = Some(resp); }
                }
            }
        }
    }

    // 插件执行后，统一应用 HeaderOp 到真实 RequestHeader
    apply_request_header_ops(ctx, head);
    apply_request_uri_patch(ctx, head);

    if respond.is_some() {
        return Ok(respond);
    }
    Ok(None)
}

/// 执行请求体阶段插件
pub async fn run_on_request_body(
    plugin_type: PluginType,
    body: &mut Option<Bytes>,
    ctx: &mut HttpContext,
) -> HandlerResult<()> {
    match plugin_type {
        PluginType::Global => {
            let plugins = GlobalPluginFactory::get_plugins();
            for plugin in plugins.iter() {
                execute_plugin_async!(plugin, ctx, on_request_body, body);
            }
        }
        PluginType::Route => {
            // SAFE：在 routing 时已经设置
            let route = ctx.get_route().unwrap();
            let plugins = &route.plugins;
            for plugin in plugins.iter() {
                execute_plugin_async!(plugin, ctx, on_request_body, body);
            }
        }
        PluginType::Model {
            model_name,
            provider_name,
        } => {
            let provider = ModelFactory::get_special_provider(&model_name, &provider_name).unwrap();
            if let Some(plugin) = provider.plugins {
                log::info!(
                    "execute model provider request converter plugin: {}",
                    plugin.name
                );
                execute_plugin_async!(plugin, ctx, on_request_body, body);
            }
        }
    }
    Ok(())
}

/// 执行响应阶段插件
pub async fn run_on_response(
    plugin_type: PluginType,
    head: &mut ResponseHeader,
    ctx: &mut HttpContext,
) -> HandlerResult<Option<Response>> {
    // 清除上一阶段的 HeaderOp
    ctx.remove_any_state::<Vec<HeaderOp>>(RESPONSE_HEADER_PATCH);

    let mut respond: Option<Response> = None;
    match plugin_type {
        PluginType::Global => {
            let plugins = GlobalPluginFactory::get_plugins();
            for plugin in plugins.iter() {
                match execute_plugin_outcome!(plugin, ctx, on_response) {
                    Outcome::Continue => continue,
                    Outcome::Respond(resp) => { respond = Some(resp); break; }
                }
            }
        }
        PluginType::Route => {
            // SAFE：在 routing 时已经设置
            let route = ctx.get_route().unwrap();
            let plugins = &route.plugins;
            for plugin in plugins.iter() {
                match execute_plugin_outcome!(plugin, ctx, on_response) {
                    Outcome::Continue => continue,
                    Outcome::Respond(resp) => { respond = Some(resp); break; }
                }
            }
        }
        PluginType::Model {
            model_name,
            provider_name,
        } => {
            let provider = ModelFactory::get_special_provider(&model_name, &provider_name).unwrap();
            if let Some(plugin) = provider.plugins {
                log::info!(
                    "execute model provider request converter plugin: {}",
                    plugin.name
                );
                match execute_plugin_outcome!(plugin, ctx, on_response) {
                    Outcome::Continue => {}
                    Outcome::Respond(resp) => { respond = Some(resp); }
                }
            }
        }
    }

    // 插件执行后，统一应用 HeaderOp 到真实 ResponseHeader
    apply_response_header_ops(ctx, head);

    if respond.is_some() {
        return Ok(respond);
    }
    Ok(None)
}

/// 执行响应体阶段插件
pub async fn run_on_response_body(
    plugin_type: PluginType,
    body: &mut Option<Bytes>,
    ctx: &mut HttpContext,
) -> HandlerResult<()> {
    match plugin_type {
        PluginType::Global => {
            let plugins = GlobalPluginFactory::get_plugins();
            for plugin in plugins.iter() {
                execute_plugin_async!(plugin, ctx, on_response_body, body);
            }
        }
        PluginType::Route => {
            // SAFE：在 routing 时已经设置
            let route = ctx.get_route().unwrap();
            let plugins = &route.plugins;
            for plugin in plugins.iter() {
                execute_plugin_async!(plugin, ctx, on_response_body, body);
            }
        }
        PluginType::Model {
            model_name,
            provider_name,
        } => {
            let provider = ModelFactory::get_special_provider(&model_name, &provider_name).unwrap();
            if let Some(plugin) = provider.plugins {
                log::info!(
                    "execute model provider request converter plugin: {}",
                    plugin.name
                );
                execute_plugin_async!(plugin, ctx, on_response_body, body);
            }
        }
    }

    Ok(())
}

/// 执行日志阶段插件
pub async fn run_on_logging(ctx: &mut HttpContext) {
    // 执行路由插件的logging
    if let Some(route) = ctx.get_route() {
        let plugins = &route.plugins;
        for plugin in plugins.iter() {
            PluginFactory::on_logging(plugin, ctx).await;
        }
    }

    // 执行模型插件的logging
    if let Some(model_name) = ctx.get_proxy_model_name()
        && let Some(provider) = ctx.get_proxy_model_provider()
    {
        let provider = ModelFactory::get_special_provider(&model_name, &provider.name).unwrap();
        if let Some(plugin) = provider.plugins {
            log::info!(
                "execute model provider request converter plugin: {}",
                plugin.name
            );
            PluginFactory::on_logging(&plugin, ctx).await;
        }
    }

    // 执行全局插件的logging
    let plugins = GlobalPluginFactory::get_plugins();
    for plugin in plugins.iter() {
        PluginFactory::on_logging(plugin, ctx).await;
    }
}

// ---------------------------------------------------------------------------
// HeaderOp 应用辅助函数
// ---------------------------------------------------------------------------

/// 将插件产生的 HeaderOp 应用到真实的 RequestHeader
fn apply_request_header_ops(ctx: &HttpContext, head: &mut RequestHeader) {
    let Some(ops) = ctx.get_any_state::<Vec<HeaderOp>>(REQUEST_HEADER_PATCH) else {
        return;
    };
    for op in ops.iter() {
        match op {
            HeaderOp::Set(name, value) => {
                head.insert_header(name.clone(), value.as_bytes().to_vec()).ok();
            }
            HeaderOp::Append(name, value) => {
                head.append_header(name.clone(), value.as_bytes().to_vec()).ok();
            }
            HeaderOp::Remove(name) => {
                head.remove_header(name);
            }
        }
    }
}

/// 将插件产生的 HeaderOp 应用到真实的 ResponseHeader
fn apply_response_header_ops(ctx: &HttpContext, head: &mut ResponseHeader) {
    let Some(ops) = ctx.get_any_state::<Vec<HeaderOp>>(RESPONSE_HEADER_PATCH) else {
        return;
    };
    for op in ops.iter() {
        match op {
            HeaderOp::Set(name, value) => {
                head.insert_header(name.clone(), value.as_bytes().to_vec()).ok();
            }
            HeaderOp::Append(name, value) => {
                head.append_header(name.clone(), value.as_bytes().to_vec()).ok();
            }
            HeaderOp::Remove(name) => {
                head.remove_header(name);
            }
        }
    }
}

/// 将插件设置的请求 URI 应用到真实的 RequestHeader（路径改写）
fn apply_request_uri_patch(ctx: &HttpContext, head: &mut RequestHeader) {
    let Some(uri) = ctx.get_any_state::<http::Uri>(REQUEST_URI_PATCH) else {
        return;
    };
    head.uri = (*uri).clone();
}
