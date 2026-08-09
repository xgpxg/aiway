//! # aiway-plugin
//!
//! 网关插件 SDK，用于实现自定义插件。

pub mod wasm_types;

mod macros;
mod plugin_ctx;
mod wasm_ctx;

pub use crate::plugin_ctx::{
    FormPart, HttpRequest, HttpRequestBuilder, HttpResponse, LOG_DEBUG, LOG_ERROR, LOG_INFO,
    LOG_TRACE, LOG_WARN, PluginContext, PluginContextExt,
};
pub use async_trait::async_trait;
pub use bincode;
pub use bytes::Bytes;
pub use http;
pub use semver::Version;
use serde::de::DeserializeOwned;
pub use serde_json;
use serde_json::Value;
pub use wasm_ctx::{respond_to_host, WasmHttpContext};

/// 插件信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginInfo {
    /// 插件版本
    pub version: Version,
    /// 默认配置
    pub default_config: Value,
    /// 插件描述，用于简要描述插件的功能
    pub description: String,
    /// 插件使用手册，通常是一个内容为 `markdown` 格式的字符串
    pub readme: Option<String>,
}

/// 插件控制流
///
/// 插件阶段返回此枚举，决定网关是继续执行后续插件/流程，还是由插件主动响应并终止。
#[derive(Debug)]
pub enum Outcome {
    /// 继续执行下一个插件或后续流程
    Continue,
    /// 主动响应，会终止后续流程，当某个插件处理后，不想继续后续流程时，返回该值
    Respond(Response),
}

/// 插件主动响应
///
/// 当插件需要直接返回响应时使用（如预检、缓存命中、mock 等场景）。
#[derive(Debug)]
pub struct Response {
    /// HTTP 状态码
    pub status: u16,
    /// 响应头
    pub headers: Vec<(String, String)>,
    /// 响应体
    pub body: Vec<u8>,
}

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
    SerdeError(String),
    /// HTTP 错误（发起HTTP调用错误）
    HttpError(String),
}

pub type PluginResult = Result<Outcome, PluginError>;

impl Outcome {
    /// 继续执行，等价于 `Ok(Outcome::Continue)`
    pub fn goon() -> PluginResult {
        Ok(Outcome::Continue)
    }

    /// 主动响应，会终止后续流程
    pub fn respond(status: u16, headers: Vec<(String, String)>, body: Vec<u8>) -> PluginResult {
        Ok(Outcome::Respond(Response {
            status,
            headers,
            body,
        }))
    }

    /// 拒绝请求，等价于 `respond(status, vec![], msg)`
    ///
    /// 用于限流(429)、鉴权失败(403)、参数校验(400) 等场景。
    pub fn reject(status: u16, msg: impl Into<String>) -> PluginResult {
        Ok(Outcome::Respond(Response {
            status,
            headers: Vec::new(),
            body: msg.into().into_bytes(),
        }))
    }

    pub fn execute_error(msg: impl Into<String>) -> PluginResult {
        Err(PluginError::ExecuteError(msg.into()))
    }

    pub fn not_found(msg: impl Into<String>) -> PluginResult {
        Err(PluginError::NotFound(msg.into()))
    }
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginError::ExecuteError(msg) => write!(f, "{}", msg),
            PluginError::NotFound(msg) => write!(f, "{}", msg),
            PluginError::LoadError(msg) => write!(f, "{}", msg),
            PluginError::SerdeError(msg) => write!(f, "{}", msg),
            PluginError::HttpError(msg) => write!(f, "{}", msg),
        }
    }
}

/// 插件定义
///
/// 插件开发者实现此 trait。
#[async_trait]
pub trait Plugin: Send + Sync {
    /// 插件名称
    fn name(&self) -> &str;
    /// 插件信息
    fn info(&self) -> PluginInfo;

    /// 请求阶段，可改写请求头
    ///
    /// 插件配置通过 `ctx.config()` 获取，请求头通过 `ctx` 读写。
    async fn on_request(&self, _ctx: &mut dyn PluginContext) -> PluginResult {
        Ok(Outcome::Continue)
    }

    /// 请求体阶段，可改写请求体
    ///
    /// 请求体通过 `ctx.request_body()` 读取、`ctx.set_request_body()` 覆盖。
    async fn on_request_body(&self, _ctx: &mut dyn PluginContext) -> PluginResult {
        Ok(Outcome::Continue)
    }

    /// 响应阶段，可改写响应头
    async fn on_response(&self, _ctx: &mut dyn PluginContext) -> PluginResult {
        Ok(Outcome::Continue)
    }

    /// 响应体阶段，可改写响应体
    ///
    /// 响应体通过 `ctx.response_body()` 读取、`ctx.set_response_body()` 覆盖。
    async fn on_response_body(&self, _ctx: &mut dyn PluginContext) -> PluginResult {
        Ok(Outcome::Continue)
    }

    /// 日志阶段
    async fn on_logging(&self, _: &mut dyn PluginContext) {}
}

pub trait PluginConfigExt: Plugin {
    /// 解析插件配置，这个只是方便调用，手动使用`serde_json`转换也可
    fn parse_config<T>(&self, config: &Value) -> Result<T, PluginError>
    where
        T: DeserializeOwned,
    {
        serde_json::from_value(config.clone()).map_err(|e| {
            PluginError::SerdeError(format!("[{}] pase plugin config error: {}", self.name(), e))
        })
    }
}

impl<T: Plugin> PluginConfigExt for T {}

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
