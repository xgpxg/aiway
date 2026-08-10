//! WASM 侧 HttpContext 实现
//!
//! 通过宿主函数访问网关的真实请求数据，而非在 WASM 内部维护独立状态。
//! 所有数据读取均委托给宿主侧的 `aiway::host_xxx` 函数。

use crate::PluginError;
use crate::plugin_ctx::{HttpRequest, HttpResponse, PluginContext};
#[cfg(feature = "model")]
use aiway_protocol::model::Provider;
use bytes::Bytes;
use http::Uri;
use std::any::Any;
// ---------------------------------------------------------------------------
// 宿主函数 FFI 声明
// ---------------------------------------------------------------------------

#[link(wasm_import_module = "aiway")]
unsafe extern "C" {
    fn host_request_id(buf_ptr: *mut u8, buf_len: i32) -> i32;
    fn host_request_ts() -> i64;
    fn host_is_sse() -> i32;
    fn host_is_websocket() -> i32;
    fn host_get_request_header(
        name_ptr: *const u8,
        name_len: i32,
        buf_ptr: *mut u8,
        buf_len: i32,
    ) -> i32;
    fn host_get_response_header(
        name_ptr: *const u8,
        name_len: i32,
        buf_ptr: *mut u8,
        buf_len: i32,
    ) -> i32;
    fn host_method(buf_ptr: *mut u8, buf_len: i32) -> i32;
    fn host_uri(buf_ptr: *mut u8, buf_len: i32) -> i32;
    fn host_set_uri(uri_ptr: *const u8, uri_len: i32);
    fn host_status() -> i32;
    fn host_get_route_name(buf_ptr: *mut u8, buf_len: i32) -> i32;
    fn host_get_routing_url(buf_ptr: *mut u8, buf_len: i32) -> i32;
    fn host_get_response_body_size() -> i64;
    fn host_set_response_body_size(size: i64);
    fn host_log(level: i32, msg_ptr: *const u8, msg_len: i32);
    #[cfg(feature = "model")]
    fn host_get_model_name(buf_ptr: *mut u8, buf_len: i32) -> i32;
    #[cfg(feature = "model")]
    fn host_get_model_provider(buf_ptr: *mut u8, buf_len: i32) -> i32;
    fn host_set_request_header(
        name_ptr: *const u8,
        name_len: i32,
        value_ptr: *const u8,
        value_len: i32,
    );
    fn host_set_response_header(
        name_ptr: *const u8,
        name_len: i32,
        value_ptr: *const u8,
        value_len: i32,
    );
    fn host_append_request_header(
        name_ptr: *const u8,
        name_len: i32,
        value_ptr: *const u8,
        value_len: i32,
    );
    fn host_append_response_header(
        name_ptr: *const u8,
        name_len: i32,
        value_ptr: *const u8,
        value_len: i32,
    );
    fn host_remove_request_header(name_ptr: *const u8, name_len: i32);
    fn host_remove_response_header(name_ptr: *const u8, name_len: i32);
    fn host_http_request(
        req_ptr: *const u8,
        req_len: i32,
        resp_buf_ptr: *mut u8,
        resp_buf_len: i32,
    ) -> i32;
    fn host_config(buf_ptr: *mut u8, buf_len: i32) -> i32;
    fn host_get_request_body(buf_ptr: *mut u8, buf_len: i32) -> i32;
    fn host_set_request_body(body_ptr: *const u8, body_len: i32);
    fn host_get_response_body(buf_ptr: *mut u8, buf_len: i32) -> i32;
    fn host_set_response_body(body_ptr: *const u8, body_len: i32);
    fn host_respond(
        status: i32,
        hdr_ptr: *const u8,
        hdr_len: i32,
        body_ptr: *const u8,
        body_len: i32,
    );
}

// ---------------------------------------------------------------------------
// WasmHttpContext
// ---------------------------------------------------------------------------

/// WASM 侧的插件上下文实现。
///
/// 不持有任何状态，所有数据通过宿主函数按需获取。
pub struct WasmHttpContext;

fn read_host_bytes_with<F>(mut call: F, initial_len: i32) -> Option<Vec<u8>>
where
    F: FnMut(*mut u8, i32) -> i32,
{
    let mut buf = vec![0u8; initial_len as usize];
    let needed = call(buf.as_mut_ptr(), initial_len);
    if needed <= 0 {
        return None;
    }
    let mut needed = needed as usize;
    if needed > buf.len() {
        buf.resize(needed, 0);
        let len = call(buf.as_mut_ptr(), buf.len() as i32);
        if len <= 0 {
            return None;
        }
        needed = (len as usize).min(buf.len());
    }
    buf.truncate(needed);
    Some(buf)
}

/// 通过宿主函数读取字符串。
///
/// `f` 为宿主函数，遵循 snprintf 语义：返回数据实际长度（可能大于 `buf_len`）。
/// `initial_len` 为初始缓冲区大小，若数据超出则自动扩容重试。
fn read_host_string(
    f: unsafe extern "C" fn(*mut u8, i32) -> i32,
    initial_len: i32,
) -> Option<String> {
    read_host_bytes(f, initial_len).map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
}

/// 通过宿主函数读取字节数组。
///
/// 语义同 [`read_host_string`]：初始缓冲区不足时自动扩容重试。
fn read_host_bytes(
    f: unsafe extern "C" fn(*mut u8, i32) -> i32,
    initial_len: i32,
) -> Option<Vec<u8>> {
    read_host_bytes_with(
        |buf_ptr, buf_len| unsafe { f(buf_ptr, buf_len) },
        initial_len,
    )
}

/// 通知宿主记录插件主动响应（由 SDK 导出宏在 `Outcome::Respond` 时调用）。
///
/// headers 以 bincode 序列化传入，Host 侧反序列化后存入 HttpContext。
pub fn respond_to_host(status: u16, headers: Vec<(String, String)>, body: Vec<u8>) {
    let headers_bytes = bincode::serialize(&headers).unwrap_or_default();
    unsafe {
        host_respond(
            status as i32,
            headers_bytes.as_ptr(),
            headers_bytes.len() as i32,
            body.as_ptr(),
            body.len() as i32,
        );
    }
}

/// 通过宿主函数读取 bincode 序列化数据并反序列化。
///
/// 语义同 [`read_host_string`]：初始缓冲区不足时自动扩容重试。
#[cfg(feature = "model")]
fn read_host_bincode<T: serde::de::DeserializeOwned>(
    f: unsafe extern "C" fn(*mut u8, i32) -> i32,
    initial_len: i32,
) -> Option<T> {
    read_host_bytes(f, initial_len).and_then(|bytes| bincode::deserialize(&bytes).ok())
}

impl PluginContext for WasmHttpContext {
    fn request_id(&self) -> String {
        read_host_string(host_request_id, 64).unwrap_or_default()
    }

    fn request_ts(&self) -> i64 {
        unsafe { host_request_ts() }
    }

    fn is_sse(&self) -> bool {
        unsafe { host_is_sse() != 0 }
    }

    fn is_websocket(&self) -> bool {
        unsafe { host_is_websocket() != 0 }
    }

    fn get_request_header(&self, name: &str) -> Option<String> {
        let name_bytes = name.as_bytes();
        read_host_bytes_with(
            |buf_ptr, buf_len| unsafe {
                host_get_request_header(
                    name_bytes.as_ptr(),
                    name_bytes.len() as i32,
                    buf_ptr,
                    buf_len,
                )
            },
            256,
        )
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
    }

    fn get_response_header(&self, name: &str) -> Option<String> {
        let name_bytes = name.as_bytes();
        read_host_bytes_with(
            |buf_ptr, buf_len| unsafe {
                host_get_response_header(
                    name_bytes.as_ptr(),
                    name_bytes.len() as i32,
                    buf_ptr,
                    buf_len,
                )
            },
            256,
        )
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
    }

    fn method(&self) -> Option<String> {
        read_host_string(host_method, 16)
    }

    fn uri(&self) -> Option<Uri> {
        read_host_string(host_uri, 512).and_then(|s| s.parse().ok())
    }

    fn set_uri(&mut self, uri: Uri) {
        let bytes = uri.to_string();
        let b = bytes.as_bytes();
        unsafe { host_set_uri(b.as_ptr(), b.len() as i32) }
    }

    fn status(&self) -> Option<u16> {
        let v = unsafe { host_status() };
        if v < 0 { None } else { Some(v as u16) }
    }

    fn get_route_name(&self) -> Option<String> {
        read_host_string(host_get_route_name, 256)
    }

    fn get_routing_url(&self) -> Option<String> {
        read_host_string(host_get_routing_url, 512)
    }

    fn get_response_body_size(&self) -> Option<i64> {
        let v = unsafe { host_get_response_body_size() };
        if v < 0 { None } else { Some(v) }
    }

    fn set_response_body_size(&mut self, size: i64) {
        unsafe { host_set_response_body_size(size) }
    }

    fn set_request_header(&mut self, name: &str, value: &str) {
        let nb = name.as_bytes();
        let vb = value.as_bytes();
        unsafe {
            host_set_request_header(nb.as_ptr(), nb.len() as i32, vb.as_ptr(), vb.len() as i32)
        }
    }

    fn set_response_header(&mut self, name: &str, value: &str) {
        let nb = name.as_bytes();
        let vb = value.as_bytes();
        unsafe {
            host_set_response_header(nb.as_ptr(), nb.len() as i32, vb.as_ptr(), vb.len() as i32)
        }
    }

    fn append_request_header(&mut self, name: &str, value: &str) {
        let nb = name.as_bytes();
        let vb = value.as_bytes();
        unsafe {
            host_append_request_header(nb.as_ptr(), nb.len() as i32, vb.as_ptr(), vb.len() as i32)
        }
    }

    fn append_response_header(&mut self, name: &str, value: &str) {
        let nb = name.as_bytes();
        let vb = value.as_bytes();
        unsafe {
            host_append_response_header(nb.as_ptr(), nb.len() as i32, vb.as_ptr(), vb.len() as i32)
        }
    }

    fn remove_request_header(&mut self, name: &str) {
        let nb = name.as_bytes();
        unsafe { host_remove_request_header(nb.as_ptr(), nb.len() as i32) }
    }

    fn remove_response_header(&mut self, name: &str) {
        let nb = name.as_bytes();
        unsafe { host_remove_response_header(nb.as_ptr(), nb.len() as i32) }
    }

    #[cfg(feature = "model")]
    fn get_model_name(&self) -> Option<String> {
        read_host_string(host_get_model_name, 256)
    }

    #[cfg(feature = "model")]
    fn get_model_provider(&self) -> Option<Provider> {
        read_host_bincode(host_get_model_provider, 512)
    }

    fn log(&self, level: i32, msg: &str) {
        let bytes = msg.as_bytes();
        unsafe { host_log(level, bytes.as_ptr(), bytes.len() as i32) }
    }

    fn http_request(&self, req: &HttpRequest) -> Result<HttpResponse, PluginError> {
        let req_bytes = bincode::serialize(req)
            .map_err(|e| PluginError::HttpError(format!("serialize request failed: {e}")))?;

        // 初始缓冲区 4KB，不足时扩容重试
        let mut buf = vec![0u8; 4096];
        loop {
            let needed = unsafe {
                host_http_request(
                    req_bytes.as_ptr(),
                    req_bytes.len() as i32,
                    buf.as_mut_ptr(),
                    buf.len() as i32,
                )
            };
            if needed < 0 {
                return Err(PluginError::HttpError(format!(
                    "http_request failed with code {needed}"
                )));
            }
            if needed == 0 {
                return Err(PluginError::HttpError(
                    "http_request returned empty response".into(),
                ));
            }
            let needed = needed as usize;
            if needed > buf.len() {
                buf.resize(needed, 0);
                continue;
            }
            return bincode::deserialize(&buf[..needed])
                .map_err(|e| PluginError::HttpError(format!("deserialize response failed: {e}")));
        }
    }

    fn config(&self) -> Option<String> {
        read_host_string(host_config, 1024)
    }

    fn request_body(&self) -> Option<Bytes> {
        read_host_bytes(host_get_request_body, 4096).map(Bytes::from)
    }

    fn set_request_body(&mut self, body: Vec<u8>) {
        unsafe { host_set_request_body(body.as_ptr(), body.len() as i32) }
    }

    fn response_body(&self) -> Option<Bytes> {
        read_host_bytes(host_get_response_body, 4096).map(Bytes::from)
    }

    fn set_response_body(&mut self, body: Vec<u8>) {
        unsafe { host_set_response_body(body.as_ptr(), body.len() as i32) }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
