use crate::components::GlobalPluginFactory;
use crate::handler::{HandlerError, HandlerResult, respond_error, respond_error_end};
use crate::model_proxy::ModelFactory;
use aiway_protocol::context::HttpContext;
use aiway_protocol::gateway::ConfiguredPlugin;
use bytes::Bytes;
use pingora::prelude::*;
use plugin_manager::PluginFactory;

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
    ($plugin:expr, $method:ident, $($arg:expr),*) => {{
        log::debug!("execute plugin: {}", $plugin.name);
        PluginFactory::$method(&$plugin, $($arg),*).await.map_err(|e| {
            log::error!("execute plugin {} error: {}", $plugin.name, e);
            HandlerError::new(502, "BadPlugin")
        })?;
    }};
}

/// 执行单个插件并处理错误（同步版本）
macro_rules! execute_plugin_sync {
    ($plugin:expr, $method:ident, $($arg:expr),*) => {{
        log::debug!("execute plugin: {}", $plugin.name);
        PluginFactory::$method(&$plugin, $($arg),*).map_err(|e| {
            log::error!("execute plugin {} error: {}", $plugin.name, e);
            HandlerError::new(502, "BadPlugin")
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
                execute_plugin_async!(plugin, on_request, head, ctx);
            }
        }
        PluginType::Route => {
            // SAFE：在 routing 时已经设置
            let route = ctx.get_route().unwrap();
            let plugins = &route.plugins;
            for plugin in plugins.iter() {
                execute_plugin_async!(plugin, on_request, head, ctx);
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
                execute_plugin_async!(plugin, on_request,head, ctx);
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
                execute_plugin_async!(plugin, on_request_body,body, ctx);
            }
        }
        PluginType::Route => {
            // SAFE：在 routing 时已经设置
            let route = ctx.get_route().unwrap();
            let plugins = &route.plugins;
            for plugin in plugins.iter() {
                execute_plugin_async!(plugin, on_request_body, body, ctx);
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
                execute_plugin_async!(plugin, on_request_body,body,ctx);
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
                execute_plugin_async!(plugin, on_response,  head, ctx);
            }
        }
        PluginType::Route => {
            // SAFE：在 routing 时已经设置
            let route = ctx.get_route().unwrap();
            let plugins = &route.plugins;
            for plugin in plugins.iter() {
                execute_plugin_async!(plugin, on_response, head, ctx);
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
                execute_plugin_async!(plugin, on_response, head, ctx);
            }
        }
    }

    Ok(())
}

pub fn run_on_response_body(
    plugin_type: PluginType,
    body: &mut Option<Bytes>,
    ctx: &mut HttpContext,
) -> HandlerResult<()> {
    match plugin_type {
        PluginType::Global => {
            let plugins = GlobalPluginFactory::get_plugins();
            for plugin in plugins.iter() {
                execute_plugin_sync!(plugin, on_response_body, body, ctx);
            }
        }
        PluginType::Route => {
            // SAFE：在 routing 时已经设置
            let route = ctx.get_route().unwrap();
            let plugins = &route.plugins;
            for plugin in plugins.iter() {
                execute_plugin_sync!(plugin, on_response_body, body, ctx);
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
                execute_plugin_sync!(plugin, on_response_body, body, ctx);
            }
        }
    }

    Ok(())
}
