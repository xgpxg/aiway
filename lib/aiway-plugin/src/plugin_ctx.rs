//! 插件上下文接口定义
//!
//! 定义插件可访问的上下文操作，宿主侧和 WASM 侧分别提供实现。

#[cfg(feature = "model")]
use aiway_protocol::model::Provider;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;

use crate::PluginError;
use aiway_protocol::context::{
    HeaderOp, HttpContext, REQUEST_HEADER_PATCH, REQUEST_URI_PATCH, RESPONSE_HEADER_PATCH,
    parts::SerdeParts,
};
use http::Uri;

/// 日志级别常量，与 WASM 侧和 Host 侧保持一致
pub const LOG_ERROR: i32 = 1;
pub const LOG_WARN: i32 = 2;
pub const LOG_INFO: i32 = 3;
pub const LOG_DEBUG: i32 = 4;
pub const LOG_TRACE: i32 = 5;

/// HTTP 请求参数
#[derive(Serialize, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
    /// URL 编码表单（与 body/multipart 互斥，优先级: multipart > form > body）
    pub form: Option<HashMap<String, String>>,
    /// Multipart 表单（与 body/form 互斥，优先级最高）
    pub multipart: Option<Vec<FormPart>>,
    pub timeout_ms: u64,
}

/// Multipart 表单字段
#[derive(Serialize, Deserialize)]
pub struct FormPart {
    pub key: String,
    /// 字段值（文本或文件内容）
    pub value: Vec<u8>,
    /// 文件名（文件上传时设置）
    pub file_name: Option<String>,
    /// MIME 类型（如 "text/plain"、"image/png"）
    pub mime_type: Option<String>,
}

/// HTTP 请求构建器
pub struct HttpRequestBuilder {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<Vec<u8>>,
    form: Option<HashMap<String, String>>,
    multipart: Option<Vec<FormPart>>,
    timeout_ms: u64,
}

impl HttpRequestBuilder {
    /// 创建构建器，`method` 和 `url` 为必填项
    pub fn new(method: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            url: url.into(),
            headers: Vec::new(),
            body: None,
            form: None,
            multipart: None,
            timeout_ms: 10_000,
        }
    }

    /// 添加请求头
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }

    /// 设置原始请求体（与 form/multipart 互斥）
    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }

    /// 设置 URL 编码表单（与 body/multipart 互斥）
    pub fn form(mut self, form: HashMap<String, String>) -> Self {
        self.form = Some(form);
        self
    }

    /// 添加单个表单字段
    pub fn add_form_field(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.form
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value.into());
        self
    }

    /// 设置 Multipart 表单字段列表（与 body/form 互斥）
    pub fn multipart(mut self, parts: Vec<FormPart>) -> Self {
        self.multipart = Some(parts);
        self
    }

    /// 添加单个 Multipart 字段
    pub fn add_multipart_part(mut self, part: FormPart) -> Self {
        self.multipart.get_or_insert_with(Vec::new).push(part);
        self
    }

    /// 设置超时时间（毫秒），默认 10000
    pub fn timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// 构建 HttpRequest
    pub fn build(self) -> HttpRequest {
        HttpRequest {
            method: self.method,
            url: self.url,
            headers: self.headers,
            body: self.body,
            form: self.form,
            multipart: self.multipart,
            timeout_ms: self.timeout_ms,
        }
    }
}

/// HTTP 响应结果
#[derive(Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// 将响应体作为 UTF-8 文本返回
    pub fn text(&self) -> Result<String, PluginError> {
        String::from_utf8(self.body.clone())
            .map_err(|e| PluginError::HttpError(format!("invalid UTF-8 response: {e}")))
    }

    /// 将响应体作为 JSON 反序列化
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, PluginError> {
        serde_json::from_slice(&self.body)
            .map_err(|e| PluginError::SerdeError(format!("JSON deserialize failed: {e}")))
    }
}

/// 插件上下文接口
///
/// 宿主侧通过 `HttpContext` 实现，WASM 侧通过 `WasmHttpContext` 实现。
/// 插件开发者面向此 trait 编程，不依赖具体实现。
pub trait PluginContext: Send {
    /// 请求 ID
    fn request_id(&self) -> String;
    /// 请求时间戳（毫秒）
    fn request_ts(&self) -> i64;
    /// 是否为 SSE 连接
    fn is_sse(&self) -> bool;
    /// 是否为 WebSocket 连接
    fn is_websocket(&self) -> bool;
    /// 获取原始请求头（从 REQUEST_RAW_PARTS 读取，跨阶段可用）
    fn get_request_header(&self, name: &str) -> Option<String>;
    /// 获取原始响应头（从 RESPONSE_SERDE_PARTS 读取，跨阶段可用）
    fn get_response_header(&self, name: &str) -> Option<String>;
    /// 请求方法（仅 on_request 阶段有值）
    fn method(&self) -> Option<String>;
    /// 请求 URI（仅 on_request 阶段有值）
    fn uri(&self) -> Option<Uri>;
    /// 覆盖写入请求 URI（仅 on_request 阶段生效，路径改写场景）
    fn set_uri(&mut self, uri: Uri);
    /// 响应状态码（仅 on_response 阶段有值）
    fn status(&self) -> Option<u16>;
    /// 路由名称
    fn get_route_name(&self) -> Option<String>;
    /// 路由目标地址
    fn get_routing_url(&self) -> Option<String>;
    /// 响应体大小
    fn get_response_body_size(&self) -> Option<i64>;
    /// 设置响应体大小
    fn set_response_body_size(&mut self, size: i64);

    /// 覆盖写入请求头
    fn set_request_header(&mut self, name: &str, value: &str);
    /// 覆盖写入响应头
    fn set_response_header(&mut self, name: &str, value: &str);
    /// 多值追加请求头
    fn append_request_header(&mut self, name: &str, value: &str);
    /// 多值追加响应头
    fn append_response_header(&mut self, name: &str, value: &str);
    /// 移除请求头
    fn remove_request_header(&mut self, name: &str);
    /// 移除响应头
    fn remove_response_header(&mut self, name: &str);
    /// 模型名称（仅模型插件可用）
    #[cfg(feature = "model")]
    fn get_model_name(&self) -> Option<String>;
    /// 模型提供商（仅模型插件可用）
    #[cfg(feature = "model")]
    fn get_model_provider(&self) -> Option<Provider>;

    /// 输出日志（底层接口，level 使用 LOG_* 常量）
    fn log(&self, level: i32, msg: &str);
    /// 输出 ERROR 级别日志
    fn log_error(&self, msg: &str) {
        self.log(LOG_ERROR, msg);
    }
    /// 输出 WARN 级别日志
    fn log_warn(&self, msg: &str) {
        self.log(LOG_WARN, msg);
    }
    /// 输出 INFO 级别日志
    fn log_info(&self, msg: &str) {
        self.log(LOG_INFO, msg);
    }
    /// 输出 DEBUG 级别日志
    fn log_debug(&self, msg: &str) {
        self.log(LOG_DEBUG, msg);
    }
    /// 输出 TRACE 级别日志
    fn log_trace(&self, msg: &str) {
        self.log(LOG_TRACE, msg);
    }

    /// 发起 HTTP 请求（默认实现返回错误，WASM 侧通过宿主函数重写）
    fn http_request(&self, _req: &HttpRequest) -> Result<HttpResponse, PluginError> {
        Err(PluginError::HttpError(
            "http_request not supported in this context".into(),
        ))
    }

    /// 类型擦除，供宿主侧 downcast 获取 `HttpContext`
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl PluginContext for HttpContext {
    fn request_id(&self) -> String {
        HttpContext::request_id(self)
    }

    fn request_ts(&self) -> i64 {
        HttpContext::request_ts(self)
    }

    fn is_sse(&self) -> bool {
        HttpContext::is_sse(self)
    }

    fn is_websocket(&self) -> bool {
        HttpContext::is_websocket(self)
    }

    fn get_request_header(&self, name: &str) -> Option<String> {
        self.get_state::<SerdeParts>(Self::REQUEST_RAW_PARTS)
            .and_then(|parts| {
                parts
                    .headers
                    .as_ref()?
                    .get(name)
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string())
            })
    }

    fn get_response_header(&self, name: &str) -> Option<String> {
        self.get_state::<SerdeParts>(Self::RESPONSE_SERDE_PARTS)
            .and_then(|parts| {
                parts
                    .headers
                    .as_ref()?
                    .get(name)
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string())
            })
    }

    fn method(&self) -> Option<String> {
        self.get_state::<SerdeParts>(Self::REQUEST_RAW_PARTS)
            .and_then(|parts| parts.method.map(|m| m.to_string()))
    }

    fn uri(&self) -> Option<Uri> {
        self.get_state::<SerdeParts>(Self::REQUEST_RAW_PARTS)
            .and_then(|parts| parts.uri)
    }

    fn set_uri(&mut self, uri: Uri) {
        self.insert_any_state(REQUEST_URI_PATCH, uri);
    }

    fn status(&self) -> Option<u16> {
        self.get_state::<SerdeParts>(Self::RESPONSE_SERDE_PARTS)
            .and_then(|parts| parts.status_code.map(|s| s.as_u16()))
    }

    fn get_route_name(&self) -> Option<String> {
        self.get_route().map(|r| r.name.clone())
    }

    fn get_routing_url(&self) -> Option<String> {
        HttpContext::get_routing_url(self).cloned()
    }

    fn get_response_body_size(&self) -> Option<i64> {
        self.get_state::<i64>(Self::RESPONSE_BODY_SIZE)
    }

    fn set_response_body_size(&mut self, size: i64) {
        self.insert_state(Self::RESPONSE_BODY_SIZE, size);
    }

    fn set_request_header(&mut self, name: &str, value: &str) {
        let mut ops = self
            .get_any_state::<Vec<HeaderOp>>(REQUEST_HEADER_PATCH)
            .map(|arc| (*arc).clone())
            .unwrap_or_default();
        ops.push(HeaderOp::Set(name.to_string(), value.to_string()));
        self.insert_any_state(REQUEST_HEADER_PATCH, ops);
    }

    fn set_response_header(&mut self, name: &str, value: &str) {
        let mut ops = self
            .get_any_state::<Vec<HeaderOp>>(RESPONSE_HEADER_PATCH)
            .map(|arc| (*arc).clone())
            .unwrap_or_default();
        ops.push(HeaderOp::Set(name.to_string(), value.to_string()));
        self.insert_any_state(RESPONSE_HEADER_PATCH, ops);
    }

    fn append_request_header(&mut self, name: &str, value: &str) {
        let mut ops = self
            .get_any_state::<Vec<HeaderOp>>(REQUEST_HEADER_PATCH)
            .map(|arc| (*arc).clone())
            .unwrap_or_default();
        ops.push(HeaderOp::Append(name.to_string(), value.to_string()));
        self.insert_any_state(REQUEST_HEADER_PATCH, ops);
    }

    fn append_response_header(&mut self, name: &str, value: &str) {
        let mut ops = self
            .get_any_state::<Vec<HeaderOp>>(RESPONSE_HEADER_PATCH)
            .map(|arc| (*arc).clone())
            .unwrap_or_default();
        ops.push(HeaderOp::Append(name.to_string(), value.to_string()));
        self.insert_any_state(RESPONSE_HEADER_PATCH, ops);
    }

    fn remove_request_header(&mut self, name: &str) {
        let mut ops = self
            .get_any_state::<Vec<HeaderOp>>(REQUEST_HEADER_PATCH)
            .map(|arc| (*arc).clone())
            .unwrap_or_default();
        ops.push(HeaderOp::Remove(name.to_string()));
        self.insert_any_state(REQUEST_HEADER_PATCH, ops);
    }

    fn remove_response_header(&mut self, name: &str) {
        let mut ops = self
            .get_any_state::<Vec<HeaderOp>>(RESPONSE_HEADER_PATCH)
            .map(|arc| (*arc).clone())
            .unwrap_or_default();
        ops.push(HeaderOp::Remove(name.to_string()));
        self.insert_any_state(RESPONSE_HEADER_PATCH, ops);
    }

    #[cfg(feature = "model")]
    fn get_model_name(&self) -> Option<String> {
        self.get_proxy_model_name()
    }

    #[cfg(feature = "model")]
    fn get_model_provider(&self) -> Option<Provider> {
        self.get_proxy_model_provider()
    }

    fn log(&self, level: i32, msg: &str) {
        match level {
            LOG_ERROR => log::error!("{}", msg),
            LOG_WARN => log::warn!("{}", msg),
            LOG_INFO => log::info!("{}", msg),
            LOG_DEBUG => log::debug!("{}", msg),
            LOG_TRACE => log::trace!("{}", msg),
            _ => log::info!("{}", msg),
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
