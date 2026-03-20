use crate::components::GlobalPluginFactory;
use crate::handler::{HttpError, HttpResult};
use crate::model_proxy::ModelFactory;
use aiway_protocol::context::HttpContext;
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

/// 执行请求阶段插件
pub async fn run_on_request(
    plugin_type: PluginType,
    head: &mut RequestHeader,
    ctx: &mut HttpContext,
) -> HttpResult<()> {
    match plugin_type {
        PluginType::Global => {
            // 处理全局的插件
            let plugins = &GlobalPluginFactory::get_plugins();
            for configured_plugin in plugins.iter() {
                log::debug!("execute global plugin: {}", configured_plugin.name);
                let result = PluginFactory::on_request(configured_plugin, head, ctx).await;
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!(
                            "execute global plugin {} error: {}",
                            configured_plugin.name,
                            e
                        );
                        return Err(HttpError::new(502, "BadPlugin"));
                    }
                }
            }
        }
        PluginType::Route => {
            // SAFE：在routing时已经设置
            let route = ctx.get_route().unwrap();
            let plugins = &route.plugins;
            for configured_plugin in plugins.iter() {
                log::debug!("execute route plugin: {}", configured_plugin.name);
                let result = PluginFactory::on_request(configured_plugin, head, ctx).await;
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!(
                            "execute route plugin {} error: {}",
                            configured_plugin.name,
                            e
                        );
                        return Err(HttpError::new(502, "BadPlugin"));
                    }
                }
            }
        }
        PluginType::Model {
            model_name,
            provider_name,
        } => {
            // 处理模型提供商的插件
            let provider = ModelFactory::get_special_provider(&model_name, &provider_name).unwrap();
            if let Some(configured_plugin) = provider.plugins {
                log::info!(
                    "execute model provider request converter plugin: {}",
                    configured_plugin.name
                );
                let result = PluginFactory::on_request(&configured_plugin, head, ctx).await;
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!(
                            "execute route pre filter plugin {} error: {}",
                            configured_plugin.name,
                            e
                        );
                        return Err(HttpError::new(502, "BadPlugin"));
                    }
                }
            }
        }
    }

    // 处理模型提供商的插件

    Ok(())
}

pub async fn run_on_request_body(
    plugin_type: PluginType,
    body: &mut Option<Bytes>,
    ctx: &mut HttpContext,
) -> HttpResult<()> {
    match plugin_type {
        PluginType::Global => {
            // 处理全局的插件
            let plugins = &GlobalPluginFactory::get_plugins();
            for configured_plugin in plugins.iter() {
                log::debug!("execute global plugin: {}", configured_plugin.name);
                let result = PluginFactory::on_request_body(configured_plugin, body, ctx).await;
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!(
                            "execute global plugin {} error: {}",
                            configured_plugin.name,
                            e
                        );
                        return Err(HttpError::new(502, "BadPlugin"));
                    }
                }
            }
        }
        PluginType::Route => {
            // SAFE：在routing时已经设置
            let route = ctx.get_route().unwrap();
            let pre_filters = &route.plugins;
            for configured_plugin in pre_filters.iter() {
                log::debug!(
                    "execute route pre filter plugin: {}",
                    configured_plugin.name
                );
                let result = PluginFactory::on_request_body(configured_plugin, body, ctx).await;
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!(
                            "execute route pre filter plugin {} error: {}",
                            configured_plugin.name,
                            e
                        );
                        return Err(HttpError::new(502, "BadPlugin"));
                    }
                }
            }
        }
        PluginType::Model {
            model_name,
            provider_name,
        } => {
            // 处理模型提供商的插件
            let provider = ModelFactory::get_special_provider(&model_name, &provider_name).unwrap();
            if let Some(configured_plugin) = provider.plugins {
                log::info!(
                    "execute model provider request converter plugin: {}",
                    configured_plugin.name
                );
                let result = PluginFactory::on_request_body(&configured_plugin, body, ctx).await;
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!(
                            "execute route pre filter plugin {} error: {}",
                            configured_plugin.name,
                            e
                        );
                        return Err(HttpError::new(502, "BadPlugin"));
                    }
                }
            }
        }
    }
    Ok(())
}

pub async fn run_on_response(
    plugin_type: PluginType,
    head: &mut ResponseHeader,
    ctx: &mut HttpContext,
) -> HttpResult<()> {
    match plugin_type {
        PluginType::Global => {
            // 处理全局的插件
            let plugins = &GlobalPluginFactory::get_plugins();
            for configured_plugin in plugins.iter() {
                log::debug!("execute global plugin: {}", configured_plugin.name);
                let result = PluginFactory::on_response(configured_plugin, head, ctx).await;
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!(
                            "execute global plugin {} error: {}",
                            configured_plugin.name,
                            e
                        );
                        return Err(HttpError::new(502, "BadPlugin"));
                    }
                }
            }
        }
        PluginType::Route => {
            // SAFE：在routing时已经设置
            let route = ctx.get_route().unwrap();
            let pre_filters = &route.plugins;
            for configured_plugin in pre_filters.iter() {
                log::debug!(
                    "execute route pre filter plugin: {}",
                    configured_plugin.name
                );
                let result = PluginFactory::on_response(configured_plugin, head, ctx).await;
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!(
                            "execute route pre filter plugin {} error: {}",
                            configured_plugin.name,
                            e
                        );
                        return Err(HttpError::new(502, "BadPlugin"));
                    }
                }
            }
        }
        PluginType::Model {
            model_name,
            provider_name,
        } => {
            // 处理模型提供商的插件
            let provider = ModelFactory::get_special_provider(&model_name, &provider_name).unwrap();
            if let Some(configured_plugin) = provider.plugins {
                log::info!(
                    "execute model provider request converter plugin: {}",
                    configured_plugin.name
                );
                let result = PluginFactory::on_response(&configured_plugin, head, ctx).await;
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!(
                            "execute route pre filter plugin {} error: {}",
                            configured_plugin.name,
                            e
                        );
                        return Err(HttpError::new(502, "BadPlugin"));
                    }
                }
            }
        }
    }

    // 处理模型提供商的插件

    Ok(())
}

pub fn run_on_response_body(
    plugin_type: PluginType,
    body: &mut Option<Bytes>,
    ctx: &mut HttpContext,
) -> HttpResult<()> {
    match plugin_type {
        PluginType::Global => {
            // 处理全局的插件
            let plugins = &GlobalPluginFactory::get_plugins();
            for configured_plugin in plugins.iter() {
                log::debug!("execute global plugin: {}", configured_plugin.name);
                let result = PluginFactory::on_response_body(configured_plugin, body, ctx);
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!(
                            "execute global plugin {} error: {}",
                            configured_plugin.name,
                            e
                        );
                        return Err(HttpError::new(502, "BadPlugin"));
                    }
                }
            }
        }
        PluginType::Route => {
            // SAFE：在routing时已经设置
            let route = ctx.get_route().unwrap();
            let pre_filters = &route.plugins;
            for configured_plugin in pre_filters.iter() {
                log::debug!(
                    "execute route pre filter plugin: {}",
                    configured_plugin.name
                );
                let result = PluginFactory::on_response_body(configured_plugin, body, ctx);
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!(
                            "execute route pre filter plugin {} error: {}",
                            configured_plugin.name,
                            e
                        );
                        return Err(HttpError::new(502, "BadPlugin"));
                    }
                }
            }
        }
        PluginType::Model {
            model_name,
            provider_name,
        } => {
            // 处理模型提供商的插件
            let provider = ModelFactory::get_special_provider(&model_name, &provider_name).unwrap();
            if let Some(configured_plugin) = provider.plugins {
                log::info!(
                    "execute model provider request converter plugin: {}",
                    configured_plugin.name
                );
                let result = PluginFactory::on_response_body(&configured_plugin, body, ctx);
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!(
                            "execute route pre filter plugin {} error: {}",
                            configured_plugin.name,
                            e
                        );
                        return Err(HttpError::new(502, "BadPlugin"));
                    }
                }
            }
        }
    }

    // 处理模型提供商的插件

    Ok(())
}
