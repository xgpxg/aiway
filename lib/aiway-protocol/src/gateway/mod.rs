//! # 网关相关协议定义
//!
//! 主要定义以下内容：
//! 1. 请求/响应上下文
//! 2. 网关与插件交互协议
//! 3. 路由配置
//! 4. 服务配置
//!

#[cfg(feature = "alert")]
pub mod alert;
#[cfg(feature = "api-key")]
mod api_key;
pub mod config;
mod firewall;
mod global_filter;
pub mod plugin;
pub mod request_log;
pub mod service;
pub mod state;

#[cfg(feature = "api-key")]
pub use api_key::ApiKey;
pub use config::Config;
pub use firewall::AllowDenyPolicy;
pub use firewall::Firewall;
pub use global_filter::GlobalFilter;
pub use plugin::ConfiguredPlugin;
pub use plugin::Plugin;
pub use service::Service;
