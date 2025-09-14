//! # 全局fairing
//!
//! faring分为两个阶段：
//! 1. 收到请求，到达API处理端点前
//! 2. API端点处理完成，响应客户端前
//!
//! 这两个阶段，通过类似Filter的实现，进行拦截处理。
//!
//! 在第一阶段，即前置处理阶段，按顺序执行已配置的插件，这些插件需要接收上一个插件的处理结果，并传递给下一个插件，即链式调用。
//! 要求插件的返回值为`Result<RequestContext>`。
//! 返回Ok：继续执行下一个插件
//! 返回Err：终止执行，修改请求的uri，强制转发到一个特殊的端点。
//!
//! 在第二阶段，即后置处理阶段，按顺序执行已配置的插件，这些插件需要接收上一个插件的处理结果，并传递给下一个插件，即链式调用。
//! 可在此阶段修改响应结果。
//! TODO 以上内容需重写
pub mod auth;
pub mod filter;
pub mod global_filter;
pub mod logger;
pub mod request;
pub mod security;
pub mod cleanup;
