//! # 路由过滤器
//!
use crate::components::PLUGINS;
use crate::handler::{HttpError, HttpResult};
use aiway_protocol::context::HttpContext;
use anyhow::{Result, bail};
use pingora::prelude::*;

pub async fn pre_filter(session: &mut Session, context: &mut HttpContext) -> HttpResult<()> {
    // SAFE：在routing时已经设置
    let route = context.get_route().unwrap();
    let pre_filters = &route.pre_filters.clone();
    for configured_plugin in pre_filters.iter() {
        log::debug!(
            "execute route pre filter plugin: {}",
            configured_plugin.name
        );
        let result = PLUGINS
            .get()
            .unwrap() // SAFE: 在启动时已经初始化
            .run_on_request(configured_plugin, session, context)
            .await;
        match result {
            Ok(_) => {}
            Err(e) => {
                log::error!(
                    "execute route pre filter plugin {} error: {}",
                    configured_plugin.name,
                    e
                );
                return Err(HttpError::new(502, "BadGateway"));
            }
        }
    }
    Ok(())
}

pub async fn post_filter(
    session: &mut Session,
    response: &mut ResponseHeader,
    context: &mut HttpContext,
) -> HttpResult<()> {
    // SAFE：在routing时已经设置
    let route = context.get_route().unwrap();
    let plugins = &route.post_filters.clone();

    for configured_plugin in plugins.iter() {
        log::debug!(
            "execute route post filter plugin: {}",
            configured_plugin.name
        );
        let result = PLUGINS
            .get()
            .unwrap() // SAFE: 在启动时已经初始化
            .run_on_response(configured_plugin, session, response, context)
            .await;
        match result {
            Ok(_) => {}
            Err(e) => {
                log::error!(
                    "execute route post filter plugin {} error: {}",
                    configured_plugin.name,
                    e
                );
                return Err(HttpError::new(502, "BadGateway"));
            }
        }
    }
    Ok(())
}
