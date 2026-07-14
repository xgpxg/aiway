//! 插件上下文接口定义
//!
//! 定义插件可访问的上下文操作，宿主侧和 WASM 侧分别提供实现。

#[cfg(feature = "model")]
use crate::model::Provider;
use std::any::Any;

use super::HttpContext;

/// 日志级别常量，与 WASM 侧和 Host 侧保持一致
pub const LOG_ERROR: i32 = 1;
pub const LOG_WARN: i32 = 2;
pub const LOG_INFO: i32 = 3;
pub const LOG_DEBUG: i32 = 4;
pub const LOG_TRACE: i32 = 5;

/// 插件上下文接口
///
/// 宿主侧通过 `HttpContext` 实现，WASM 侧通过 `WasmHttpContext` 实现。
/// 插件开发者面向此 trait 编程，不依赖具体实现。
pub trait PluginContext: Send {
    /// 请求 ID
    fn request_id(&self) -> String;
    /// 请求时间戳（毫秒）
    fn request_ts(&self) -> i64;
    /// 是否为 SSE 连接
    fn is_sse(&self) -> bool;
    /// 是否为 WebSocket 连接
    fn is_websocket(&self) -> bool;
    /// 路由名称
    fn get_route_name(&self) -> Option<String>;
    /// 路由目标地址
    fn get_routing_url(&self) -> Option<String>;
    /// 响应体大小
    fn get_response_body_size(&self) -> Option<i64>;
    /// 设置响应体大小
    fn set_response_body_size(&mut self, size: i64);
    /// 模型名称（仅模型插件可用）
    #[cfg(feature = "model")]
    fn get_model_name(&self) -> Option<String>;
    /// 模型提供商（仅模型插件可用）
    #[cfg(feature = "model")]
    fn get_model_provider(&self) -> Option<Provider>;

    /// 输出日志（底层接口，level 使用 LOG_* 常量）
    fn log(&self, level: i32, msg: &str);
    /// 输出 ERROR 级别日志
    fn log_error(&self, msg: &str) { self.log(LOG_ERROR, msg); }
    /// 输出 WARN 级别日志
    fn log_warn(&self, msg: &str) { self.log(LOG_WARN, msg); }
    /// 输出 INFO 级别日志
    fn log_info(&self, msg: &str) { self.log(LOG_INFO, msg); }
    /// 输出 DEBUG 级别日志
    fn log_debug(&self, msg: &str) { self.log(LOG_DEBUG, msg); }
    /// 输出 TRACE 级别日志
    fn log_trace(&self, msg: &str) { self.log(LOG_TRACE, msg); }

    /// 类型擦除，供宿主侧 downcast 获取 `HttpContext`
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl PluginContext for HttpContext {
    fn request_id(&self) -> String {
        HttpContext::request_id(self)
    }

    fn request_ts(&self) -> i64 {
        HttpContext::request_ts(self)
    }

    fn is_sse(&self) -> bool {
        HttpContext::is_sse(self)
    }

    fn is_websocket(&self) -> bool {
        HttpContext::is_websocket(self)
    }

    fn get_route_name(&self) -> Option<String> {
        self.get_route().map(|r| r.name.clone())
    }

    fn get_routing_url(&self) -> Option<String> {
        HttpContext::get_routing_url(self).cloned()
    }

    fn get_response_body_size(&self) -> Option<i64> {
        self.get_state::<i64>(Self::RESPONSE_BODY_SIZE)
    }

    fn set_response_body_size(&mut self, size: i64) {
        self.insert_state(Self::RESPONSE_BODY_SIZE, size);
    }

    #[cfg(feature = "model")]
    fn get_model_name(&self) -> Option<String> {
        self.get_proxy_model_name()
    }

    #[cfg(feature = "model")]
    fn get_model_provider(&self) -> Option<Provider> {
        self.get_proxy_model_provider()
    }

    fn log(&self, level: i32, msg: &str) {
        match level {
            LOG_ERROR => log::error!("{}", msg),
            LOG_WARN => log::warn!("{}", msg),
            LOG_INFO => log::info!("{}", msg),
            LOG_DEBUG => log::debug!("{}", msg),
            LOG_TRACE => log::trace!("{}", msg),
            _ => log::info!("{}", msg),
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
