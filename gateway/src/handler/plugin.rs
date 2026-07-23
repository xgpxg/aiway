use crate::components::GlobalPluginFactory;
use crate::handler::{HandlerError, HandlerResult};
use crate::model_proxy::ModelFactory;
use aiway_protocol::context::HttpContext;
use bytes::Bytes;
use pingora::prelude::*;
use plugin_manager::{PluginError, PluginFactory};

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
    ($plugin:expr, $ctx:expr, $method:ident, $($arg:expr),*) => {{
        log::debug!("execute plugin: {}", $plugin.name);
        PluginFactory::$method(&$plugin, $ctx, $($arg),*).await.map_err(|e| {
            match e {
                PluginError::Reject(status, message) => {
                    // 插件主动拒绝请求，透传状态码
                    HandlerError::new(status, &message)
                }
                e => {
                    // 插件自身异常，统一 502
                    log::error!("execute plugin {} error: {}", $plugin.name, e);
                    HandlerError::new(502, "BadPlugin")
                }
            }
        })?;
    }};
}

/// 执行请求阶段插件
pub async fn run_on_request(
    plugin_type: PluginType,
    head: &mut RequestHeader,
    ctx: &mut HttpContext,
) -> HandlerResult<()> {
    match plugin_type {
        PluginType::Global => {
            let plugins = GlobalPluginFactory::get_plugins();
            for plugin in plugins.iter() {
                execute_plugin_async!(plugin, ctx, on_request, head);
            }
        }
        PluginType::Route => {
            // SAFE：在 routing 时已经设置
            let route = ctx.get_route().unwrap();
            let plugins = &route.plugins;
            for plugin in plugins.iter() {
                execute_plugin_async!(plugin, ctx, on_request, head);
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
                execute_plugin_async!(plugin, ctx, on_request, head);
            }
        }
    }

    Ok(())
}

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

pub async fn run_on_response(
    plugin_type: PluginType,
    head: &mut ResponseHeader,
    ctx: &mut HttpContext,
) -> HandlerResult<()> {
    match plugin_type {
        PluginType::Global => {
            let plugins = GlobalPluginFactory::get_plugins();
            for plugin in plugins.iter() {
                execute_plugin_async!(plugin, ctx, on_response, head);
            }
        }
        PluginType::Route => {
            // SAFE：在 routing 时已经设置
            let route = ctx.get_route().unwrap();
            let plugins = &route.plugins;
            for plugin in plugins.iter() {
                execute_plugin_async!(plugin, ctx, on_response, head);
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
                execute_plugin_async!(plugin, ctx, on_response, head);
            }
        }
    }

    Ok(())
}

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
