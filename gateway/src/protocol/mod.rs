//! # 协议层
//!
//! 定义 `ProtocolHandler` trait 和 `ProtocolRegistry`，
//! 将协议分发从 `service.rs` 的 if-else 链抽象为可插拔的处理器注册。

use aiway_protocol::common::constants::{A2A_API_PREFIX, MCP_API_PREFIX, MODEL_API_PREFIX};
use aiway_protocol::context::HttpContext;
use pingora::prelude::Session;
use pingora::Error;
use plugin_manager::async_trait;
use std::sync::OnceLock;

/// 协议处理器 trait
///
/// 每个协议（MCP、Model、A2A 等）实现此接口，由 `ProtocolRegistry` 统一管理和分发。
#[async_trait]
pub trait ProtocolHandler: Send + Sync {
    /// 路径前缀，如 `"/v1/mcp/"`
    fn path_prefix(&self) -> &'static str;
    /// 协议名称，用于日志和监控
    fn name(&self) -> &'static str;
    /// 处理请求
    ///
    /// - 返回 `Ok(true)` 表示已处理
    /// - 返回 `Ok(false)` 表示继续走代理流程
    async fn handle_request(
        &self,
        session: &mut Session,
        path: &str,
        ctx: &mut HttpContext,
    ) -> pingora::Result<bool, Box<Error>>;
}

/// 协议注册表
///
/// 管理所有注册的协议处理器，按路径前缀分发请求。
pub struct ProtocolRegistry {
    handlers: Vec<Box<dyn ProtocolHandler>>,
}

impl ProtocolRegistry {
    pub fn new() -> Self {
        Self { handlers: Vec::new() }
    }

    pub fn register(&mut self, handler: Box<dyn ProtocolHandler>) {
        log::info!(
            "Registered protocol handler: {} (prefix: {})",
            handler.name(),
            handler.path_prefix()
        );
        self.handlers.push(handler);
    }

    /// 按路径前缀分发请求
    ///
    /// - 返回 `Ok(Some(handled))` 表示有处理器匹配并处理了请求
    /// - 返回 `Ok(None)` 表示没有处理器匹配
    pub async fn dispatch(
        &self,
        session: &mut Session,
        path: &str,
        ctx: &mut HttpContext,
    ) -> pingora::Result<Option<bool>, Box<Error>> {
        for handler in &self.handlers {
            if path.starts_with(handler.path_prefix()) {
                let handled = handler.handle_request(session, path, ctx).await?;
                return Ok(Some(handled));
            }
        }
        Ok(None)
    }
}

pub static REGISTRY: OnceLock<ProtocolRegistry> = OnceLock::new();

/// 初始化协议注册表
pub fn init_registry() {
    let mut registry = ProtocolRegistry::new();

    #[cfg(feature = "mcp-proxy")]
    registry.register(Box::new(McpProtocol));

    #[cfg(feature = "model-proxy")]
    registry.register(Box::new(ModelProtocol));

    #[cfg(feature = "a2a-proxy")]
    registry.register(Box::new(A2aProtocol));

    REGISTRY
        .set(registry)
        .unwrap_or_else(|_| panic!("ProtocolRegistry already initialized"));
}

// ── 内置协议处理器 ───────────────────────────────────────────────

#[cfg(feature = "mcp-proxy")]
pub struct McpProtocol;

#[cfg(feature = "mcp-proxy")]
#[async_trait]
impl ProtocolHandler for McpProtocol {
    fn path_prefix(&self) -> &'static str { MCP_API_PREFIX }
    fn name(&self) -> &'static str { "mcp" }

    async fn handle_request(
        &self,
        session: &mut Session,
        path: &str,
        _ctx: &mut HttpContext,
    ) -> pingora::Result<bool, Box<Error>> {
        crate::mcp_proxy::handle_mcp_request(session, path).await
    }
}

#[cfg(feature = "model-proxy")]
pub struct ModelProtocol;

#[cfg(feature = "model-proxy")]
#[async_trait]
impl ProtocolHandler for ModelProtocol {
    fn path_prefix(&self) -> &'static str { MODEL_API_PREFIX }
    fn name(&self) -> &'static str { "model" }

    async fn handle_request(
        &self,
        session: &mut Session,
        path: &str,
        ctx: &mut HttpContext,
    ) -> pingora::Result<bool, Box<Error>> {
        crate::model_proxy::handle_model_request(session, path, ctx).await
    }
}

#[cfg(feature = "a2a-proxy")]
pub struct A2aProtocol;

#[cfg(feature = "a2a-proxy")]
#[async_trait]
impl ProtocolHandler for A2aProtocol {
    fn path_prefix(&self) -> &'static str { A2A_API_PREFIX }
    fn name(&self) -> &'static str { "a2a" }

    async fn handle_request(
        &self,
        session: &mut Session,
        path: &str,
        _ctx: &mut HttpContext,
    ) -> pingora::Result<bool, Box<Error>> {
        crate::a2a_proxy::handle_a2a_request(session, path).await
    }
}
