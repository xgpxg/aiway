//! # 全局fairing
//!
//! faring分为两个阶段：
//! 1. 收到请求，到达API处理端点前
//! 2. API端点处理完成，响应客户端前
//!
//! 这两个阶段，通过Filter的实现扩展，进行拦截处理。
//!
//! 在第一阶段，即前置处理阶段，按顺序执行已配置的插件，并传递给下一个插件。
//! 返回Ok：继续执行下一个插件
//! 返回Err：终止执行，修改请求的uri，强制转发到一个特殊的端点。（或者考虑由配置决定是否终止流程）
//!
//! 在第二阶段，即后置处理阶段，按顺序执行已配置的插件，并传递给下一个插件。
//! 可在此阶段修改响应结果。
//!
//! ## on_request 和 on_response的执行顺序
//! - fairing按照attach的顺序依次执行
//! - 如果一个fairing同时实现on_request和on_response，则在on_request和on_response也按顺序执行
//!
//! 例如：A(Req,Res) -> B(Req) -> C(Res)
//! 则执行顺序为：A(Req) -> B(Req) -> A(Res) -> C(Res)
//!
pub mod auth;
pub mod catchers;
pub mod cleanup;
pub mod filter;
pub mod global_filter;
pub mod lb;
pub mod logger;
pub mod pre;
pub mod request;
pub mod response;
pub mod routing;
pub mod security;

