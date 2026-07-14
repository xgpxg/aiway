//! WASM 侧 HttpContext 实现
//!
//! 通过宿主函数访问网关的真实请求数据，而非在 WASM 内部维护独立状态。
//! 所有数据读取均委托给宿主侧的 `aiway::host_xxx` 函数。

use crate::protocol::context::PluginContext;
#[cfg(feature = "model")]
use crate::protocol::model::Provider;
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
    fn host_get_route_name(buf_ptr: *mut u8, buf_len: i32) -> i32;
    fn host_get_routing_url(buf_ptr: *mut u8, buf_len: i32) -> i32;
    fn host_get_response_body_size() -> i64;
    fn host_set_response_body_size(size: i64);
    fn host_log(level: i32, msg_ptr: *const u8, msg_len: i32);
    #[cfg(feature = "model")]
    fn host_get_model_name(buf_ptr: *mut u8, buf_len: i32) -> i32;
    #[cfg(feature = "model")]
    fn host_get_model_provider(buf_ptr: *mut u8, buf_len: i32) -> i32;
}

// ---------------------------------------------------------------------------
// WasmHttpContext
// ---------------------------------------------------------------------------

/// WASM 侧的插件上下文实现。
///
/// 不持有任何状态，所有数据通过宿主函数按需获取。
pub struct WasmHttpContext;

/// 通过宿主函数读取字符串。
///
/// `f` 为宿主函数，遵循 snprintf 语义：返回数据实际长度（可能大于 `buf_len`）。
/// `initial_len` 为初始缓冲区大小，若数据超出则自动扩容重试。
fn read_host_string(f: unsafe extern "C" fn(*mut u8, i32) -> i32, initial_len: i32) -> Option<String> {
    let mut buf = vec![0u8; initial_len as usize];
    let needed = unsafe { f(buf.as_mut_ptr(), initial_len) };
    if needed <= 0 {
        return None;
    }
    let needed = needed as usize;
    if needed > buf.len() {
        buf.resize(needed, 0);
        let len = unsafe { f(buf.as_mut_ptr(), needed as i32) };
        if len <= 0 {
            return None;
        }
        return Some(String::from_utf8_lossy(&buf[..len as usize]).to_string());
    }
    Some(String::from_utf8_lossy(&buf[..needed]).to_string())
}

/// 通过宿主函数读取 bincode 序列化数据并反序列化。
///
/// 语义同 [`read_host_string`]：初始缓冲区不足时自动扩容重试。
#[cfg(feature = "model")]
fn read_host_bincode<T: serde::de::DeserializeOwned>(
    f: unsafe extern "C" fn(*mut u8, i32) -> i32,
    initial_len: i32,
) -> Option<T> {
    let mut buf = vec![0u8; initial_len as usize];
    let needed = unsafe { f(buf.as_mut_ptr(), initial_len) };
    if needed <= 0 {
        return None;
    }
    let needed = needed as usize;
    if needed > buf.len() {
        buf.resize(needed, 0);
        let len = unsafe { f(buf.as_mut_ptr(), needed as i32) };
        if len <= 0 {
            return None;
        }
        return bincode::deserialize(&buf[..len as usize]).ok();
    }
    bincode::deserialize(&buf[..needed]).ok()
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

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
