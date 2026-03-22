//! # 全局过滤器
//!
use crate::components::{GLOBAL_FILTER};
use crate::handler::{HttpError, HttpResult};
use aiway_protocol::context::HttpContext;
use anyhow::{Result, bail};
use pingora::prelude::*;
use plugin_manager::PluginFactory;

pub async fn global_pre_filter(session: &mut Session, context: &mut HttpContext) -> HttpResult<()> {
    let config = GLOBAL_FILTER
        .get()
        .ok_or(HttpError::new(500, "global_filter not init"))?
        .config
        .read()
        .await;
    let plugins = &config.pre_filters;

    for configured_plugin in plugins.iter() {
        log::debug!(
            "execute global pre filter plugin: {}",
            configured_plugin.name
        );
        let result = PluginFactory::on_request(configured_plugin, session.req_header_mut(), context)
            .await;
        match result {
            Ok(_) => {}
            Err(e) => {
                log::error!(
                    "execute global pre filter plugin {} error: {}",
                    configured_plugin.name,
                    e
                );
                return Err(HttpError::new(502, "BadGateway"));
            }
        }
    }
    Ok(())
}

pub async fn global_post_filter(
    session: &mut Session,
    response: &mut ResponseHeader,
    context: &mut HttpContext,
) -> HttpResult<()> {
    let config = GLOBAL_FILTER.get().unwrap().config.read().await;
    let plugins = &config.post_filters;

    for configured_plugin in plugins.iter() {
        log::debug!(
            "execute global post filter plugin: {}",
            configured_plugin.name
        );
        let result = PluginFactory::on_response(configured_plugin, response,  context)
            .await;
        match result {
            Ok(_) => {}
            Err(e) => {
                log::error!(
                    "execute global post filter plugin {} error: {}",
                    configured_plugin.name,
                    e
                );
                return Err(HttpError::new(502, "BadGateway"));
            }
        }
    }
    Ok(())
}
