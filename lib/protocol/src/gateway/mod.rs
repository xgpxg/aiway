//! # 网关相关协议定义
//!
//! 主要定义以下内容：
//! 1. 请求/响应上下文
//! 2. 网关与插件交互协议
//! 3. 路由配置
//! 4. 服务配置
//!

#[cfg(feature = "api-key")]
mod api_key;
mod configuration;
pub mod http_context;
pub mod plugin;
pub mod request_context;
pub mod response_context;
pub mod route;
pub mod service;
pub mod state;

#[cfg(feature = "api-key")]
pub use api_key::ApiKey;
pub use configuration::Configuration;
pub use http_context::HttpContext;
pub use plugin::ConfiguredPlugin;
pub use plugin::Plugin;
pub use request_context::RequestContext;
pub use response_context::ResponseContext;
pub use route::Route;
pub use service::Service;
