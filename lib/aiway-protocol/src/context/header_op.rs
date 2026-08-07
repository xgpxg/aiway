//! Header 操作日志
//!
//! 插件通过 PluginContext 方法对请求/响应头的改写操作被记录为 HeaderOp 列表，
//! 由 Host 在插件执行结束后统一应用到 pingora RequestHeader/ResponseHeader。

use serde::{Deserialize, Serialize};

/// Header 操作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HeaderOp {
    /// 覆盖写入（移除同名已有值，插入新值）
    Set(String, String),
    /// 多值追加（保留同名已有值，追加新值）
    Append(String, String),
    /// 移除（移除所有同名值）
    Remove(String),
}

/// 请求头 patch 在 HttpContext 中的 key
pub const REQUEST_HEADER_PATCH: &str = ":req:header_patch";
/// 响应头 patch 在 HttpContext 中的 key
pub const RESPONSE_HEADER_PATCH: &str = ":resp:header_patch";
/// 请求 URI patch 在 HttpContext 中的 key（单值，`Option<Uri>`）
pub const REQUEST_URI_PATCH: &str = ":req:uri_patch";
