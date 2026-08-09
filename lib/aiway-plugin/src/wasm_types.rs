//! WASM 插件边界序列化类型
//!
//! 定义 Host（网关）与 WASM 插件之间数据交换的格式。
//!
//! 插件输入（config/body）与输出（主动响应）均通过宿主函数从 HttpContext
//! 上下文存取，边界上不再传递数据结构体，仅保留插件元信息结构。
//!
//! # `aiway_call` 返回协议
//!
//! `aiway_call(hook_id) -> i64`，返回值编码：
//! - 高 32 位：状态标记（`0` = 成功，非 `0` = 错误）
//! - 低 32 位：成功时为控制流（[`HookControl`]）；错误时为错误信息长度（字符串写入固定地址 [`ERROR_BUF_PTR`]）

use serde::{Deserialize, Serialize};

/// 插件钩子 ID
pub const HOOK_ON_REQUEST: i32 = 1;
pub const HOOK_ON_REQUEST_BODY: i32 = 2;
pub const HOOK_ON_RESPONSE: i32 = 3;
pub const HOOK_ON_RESPONSE_BODY: i32 = 4;
pub const HOOK_ON_LOGGING: i32 = 5;

/// 插件钩子执行控制流
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum HookControl {
    /// 继续执行后续插件/流程
    Continue = 0,
    /// 主动响应（数据已通过 `host_respond` 写入上下文）
    Respond = 1,
}

/// 插件错误信息在 WASM 线性内存中的固定写入地址
///
/// `aiway_call` 错误时，错误信息写入该地址，低 32 位为长度。
pub const ERROR_BUF_PTR: u32 = 1;

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
