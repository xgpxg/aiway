//! WASM 插件边界序列化类型
//!
//! 定义 Host（网关）与 WASM 插件之间数据交换的格式。
//! 使用 bincode 序列化，性能优于 JSON。

use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 插件钩子 ID
pub const HOOK_ON_REQUEST: i32 = 1;
pub const HOOK_ON_REQUEST_BODY: i32 = 2;
pub const HOOK_ON_RESPONSE: i32 = 3;
pub const HOOK_ON_RESPONSE_BODY: i32 = 4;
pub const HOOK_ON_LOGGING: i32 = 5;

/// 传递给 WASM 插件的输入数据
#[derive(Serialize, Deserialize)]
pub struct WasmInput {
    /// 插件配置（JSON 字符串）
    pub config: Arc<str>,
    /// Body 数据
    pub body: Option<Vec<u8>>,
    /// 请求 ID（仅 logging 阶段使用）
    pub request_id: Option<String>,
    /// 请求时间戳（仅 logging 阶段使用）
    pub request_ts: Option<i64>,
}

/// WASM 插件返回的输出数据
#[derive(Serialize, Deserialize)]
pub struct WasmOutput {
    /// 修改后的 Body（None 表示不修改）
    pub body: Option<Vec<u8>>,
    /// 插件主动响应（Some 时网关应终止后续流程并返回此响应）
    /// 目前近 `on_request` 和 `on_response` 阶段有效
    pub respond: Option<WasmRespond>,
}

/// 插件主动响应数据
#[derive(Serialize, Deserialize)]
pub struct WasmRespond {
    /// HTTP 状态码
    pub status: u16,
    /// 响应头
    pub headers: Vec<(String, String)>,
    /// 响应体
    pub body: Vec<u8>,
}

/// WASM 插件元信息（由 plugin_info 导出返回）
#[derive(Serialize, Deserialize)]
pub struct WasmPluginInfo {
    /// 插件名称
    pub name: String,
    /// 插件版本
    pub version: String,
    /// 插件描述
    pub description: String,
    /// 默认配置（JSON 字符串）
    pub default_config: String,
    /// 插件文档
    pub readme: Option<String>,
}
