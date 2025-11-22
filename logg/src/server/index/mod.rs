//! # 日志索引
//! 每个模块对应不同的index，用于适配quickwit。
//!
//! - aiway_logs: 通用日志索引
//! - request_logs: 网关请求日志索引
//!
pub(crate) mod aiway_logs;
pub(crate) mod request_logs;
