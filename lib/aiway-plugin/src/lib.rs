//! # aiway-plugin
//!
//! 网关插件 SDK，定义插件接口和数据交换类型。
//!
//! ## 架构
//! - **插件侧**：插件开发者依赖此 crate，实现 `Plugin` trait，使用 `export_wasm!` 宏导出 WASM
//! - **宿主侧**：`plugin-manager` crate 提供 wasmtime 运行时，加载并执行 WASM 插件
//!
//! ## 数据交换
//! Host 与 WASM 之间通过 bincode 序列化传递数据，定义在 [`wasm_types`] 模块中。

pub mod wasm_types;

mod macros;
mod wasm_ctx;

pub use aiway_protocol as protocol;
use aiway_protocol::context::PluginContext;
use aiway_protocol::context::http::{request, response};
pub use async_trait::async_trait;
pub use bytes::Bytes;
pub use http;
pub use semver::Version;
pub use serde_json;
use serde_json::Value;
pub use wasm_ctx::WasmHttpContext;

/// 插件错误类型
#[derive(Debug)]
pub enum PluginError {
    /// 执行插件业务逻辑时的错误
    ExecuteError(String),
    /// 插件不存在
    NotFound(String),
    /// 从磁盘或网络加载插件时错误
    LoadError(String),
    /// 序列化/反序列化错误
    SerializeError(String),
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginError::ExecuteError(msg) => write!(f, "{}", msg),
            PluginError::NotFound(msg) => write!(f, "{}", msg),
            PluginError::LoadError(msg) => write!(f, "{}", msg),
            PluginError::SerializeError(msg) => write!(f, "{}", msg),
        }
    }
}

/// 插件接口 trait
///
/// 插件开发者实现此 trait，宿主侧通过 `plugin-manager` 调用。
/// 上下文参数使用 `&mut dyn PluginContext`，宿主侧和 WASM 侧分别有不同实现。
#[async_trait]
pub trait Plugin: Send + Sync {
    /// 插件名称
    fn name(&self) -> &str;
    /// 插件信息
    fn info(&self) -> PluginInfo;

    /// 请求阶段，可改写头部
    async fn on_request(
        &self,
        _config: &Value,
        _head: &mut request::Parts,
        _ctx: &mut dyn PluginContext,
    ) -> Result<(), PluginError> {
        Ok(())
    }

    /// 请求体阶段，可改写请求体
    async fn on_request_body(
        &self,
        _config: &Value,
        _body: &mut Option<Bytes>,
        _ctx: &mut dyn PluginContext,
    ) -> Result<(), PluginError> {
        Ok(())
    }

    /// 响应阶段，可改写头部
    async fn on_response(
        &self,
        _config: &Value,
        _head: &mut response::Parts,
        _ctx: &mut dyn PluginContext,
    ) -> Result<(), PluginError> {
        Ok(())
    }

    /// 响应体阶段，可改写响应体（同步）
    fn on_response_body(
        &self,
        _config: &Value,
        _body: &mut Option<Bytes>,
        _ctx: &mut dyn PluginContext,
    ) -> Result<(), PluginError> {
        Ok(())
    }

    /// 日志阶段
    async fn on_logging(&self, _: &Value, _: &mut dyn PluginContext) {}
}

/// 插件信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginInfo {
    /// 插件版本
    pub version: Version,
    /// 默认配置
    pub default_config: Value,
    /// 描述
    pub description: String,
}

/// 简易 block_on，用于在同步上下文（WASM 内部）中执行 async 函数。
///
/// WASM 环境无真正异步 I/O，插件 future 必须立即返回 `Ready`。
/// 若返回 `Pending` 说明插件误用了异步 I/O（如网络请求），直接 panic 露问题。
pub fn block_on<F: Future>(f: F) -> F::Output {
    use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

    fn noop_clone(_: *const ()) -> RawWaker {
        RawWaker::new(std::ptr::null(), &VTABLE)
    }
    fn noop(_: *const ()) {}
    static VTABLE: RawWakerVTable = RawWakerVTable::new(noop_clone, noop, noop, noop);

    let raw_waker = RawWaker::new(std::ptr::null(), &VTABLE);
    let waker = unsafe { Waker::from_raw(raw_waker) };
    let mut cx = Context::from_waker(&waker);

    let mut f = std::pin::pin!(f);
    match f.as_mut().poll(&mut cx) {
        Poll::Ready(val) => val,
        Poll::Pending => panic!(
            "plugin future returned Pending in WASM context; \
             WASM plugins must not perform real async I/O"
        ),
    }
}
