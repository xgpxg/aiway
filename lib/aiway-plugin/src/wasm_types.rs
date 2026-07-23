//! WASM 插件边界序列化类型
//!
//! 定义 Host（网关）与 WASM 插件之间数据交换的格式。
//! 使用 bincode 序列化，性能优于 JSON。

use crate::http::{self, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
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
    /// HTTP 头部信息（请求或响应）
    pub head: Option<WasmHead>,
    /// Body 数据
    pub body: Option<Vec<u8>>,
    /// 请求 ID（仅 logging 阶段使用）
    pub request_id: Option<String>,
    /// 请求时间戳（仅 logging 阶段使用）
    pub request_ts: Option<i64>,
}

/// HTTP 头部信息的可序列化表示
#[derive(Serialize, Deserialize)]
pub struct WasmHead {
    /// 请求方法（仅请求阶段）
    pub method: Option<String>,
    /// 请求 URI（仅请求阶段）
    pub uri: Option<String>,
    /// 响应状态码（仅响应阶段）
    pub status: Option<u16>,
    /// 头部键值对列表
    pub headers: Vec<(String, String)>,
}

impl WasmHead {
    /// 从请求 Parts 构建
    pub fn from_request_parts(parts: &http::request::Parts) -> Self {
        let headers = parts
            .headers
            .iter()
            .filter_map(|(k, v)| v.to_str().ok().map(|v| (k.to_string(), v.to_string())))
            .collect();

        Self {
            method: Some(parts.method.to_string()),
            uri: Some(parts.uri.to_string()),
            status: None,
            headers,
        }
    }

    /// 从响应 Parts 构建
    pub fn from_response_parts(parts: &http::response::Parts) -> Self {
        let headers = parts
            .headers
            .iter()
            .filter_map(|(k, v)| v.to_str().ok().map(|v| (k.to_string(), v.to_string())))
            .collect();

        Self {
            method: None,
            uri: None,
            status: Some(parts.status.as_u16()),
            headers,
        }
    }

    /// 将修改应用到请求 Parts
    pub fn apply_to_request_parts(&self, parts: &mut http::request::Parts) {
        if let Some(ref method) = self.method
            && let Ok(m) = method.parse()
        {
            parts.method = m;
        }
        if let Some(ref uri) = self.uri
            && let Ok(u) = uri.parse()
        {
            parts.uri = u;
        }
        apply_headers(&self.headers, &mut parts.headers);
    }

    /// 将修改应用到响应 Parts
    pub fn apply_to_response_parts(&self, parts: &mut http::response::Parts) {
        if let Some(status) = self.status && let Ok(s) = http::StatusCode::from_u16(status) {
                parts.status = s;
            }
        apply_headers(&self.headers, &mut parts.headers);
    }
}

/// 将 WasmHead 中的 headers 应用到 http::HeaderMap
fn apply_headers(wasm_headers: &[(String, String)], header_map: &mut http::HeaderMap) {
    header_map.clear();
    for (k, v) in wasm_headers {
        if let (Ok(name), Ok(value)) = (HeaderName::from_str(k), HeaderValue::from_str(v)) {
            header_map.insert(name, value);
        }
    }
}

/// WASM 插件返回的输出数据
#[derive(Serialize, Deserialize)]
pub struct WasmOutput {
    /// 修改后的 HTTP 头部（None 表示不修改）
    pub head: Option<WasmHead>,
    /// 修改后的 Body（None 表示不修改）
    pub body: Option<Vec<u8>>,
}

/// WASM 插件元信息（由 plugin_info 导出返回）
#[derive(Serialize, Deserialize)]
pub struct WasmPluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    /// 默认配置（JSON 字符串）
    pub default_config: String,
    /// 插件文档
    pub readme: Option<String>,
}
