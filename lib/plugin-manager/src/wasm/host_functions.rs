//! WASM 宿主函数注册与实现
//!
//! 所有 `aiway::` 模块下的宿主函数在此文件中定义。
//! WASM 插件通过 FFI 调用这些函数，访问网关的真实 HttpContext 数据。
//!
//! ## 安全约束
//! 宿主函数通过 `Caller` 中的裸指针访问 `HttpContext`。
//! 指针在 `call_wasm` 前注入、调用后清除，期间当前线程被阻塞或独占，
//! 因此指针始终有效且不存在数据竞争。

use super::WasmStoreCtx;
use super::network::NETWORK;
use aiway_plugin::{
    HttpRequest, HttpResponse, LOG_DEBUG, LOG_ERROR, LOG_INFO, LOG_TRACE, LOG_WARN,
};
use aiway_protocol::context::HttpContext;
use mime;
use std::future::Future;
use std::time::Duration;
use wasmtime::{Caller, Linker};

/// 注册所有 `aiway::` 宿主函数到 Linker
pub fn register(linker: &mut Linker<WasmStoreCtx>) -> Result<(), crate::wasm::PluginError> {
    linker
        .func_wrap("aiway", "host_request_id", host_request_id)
        .map_err(|e| {
            crate::wasm::PluginError::LoadError(format!("register host_request_id: {e}"))
        })?;
    linker
        .func_wrap("aiway", "host_request_ts", host_request_ts)
        .map_err(|e| {
            crate::wasm::PluginError::LoadError(format!("register host_request_ts: {e}"))
        })?;
    linker
        .func_wrap("aiway", "host_is_sse", host_is_sse)
        .map_err(|e| crate::wasm::PluginError::LoadError(format!("register host_is_sse: {e}")))?;
    linker
        .func_wrap("aiway", "host_is_websocket", host_is_websocket)
        .map_err(|e| {
            crate::wasm::PluginError::LoadError(format!("register host_is_websocket: {e}"))
        })?;
    linker
        .func_wrap("aiway", "host_get_route_name", host_get_route_name)
        .map_err(|e| {
            crate::wasm::PluginError::LoadError(format!("register host_get_route_name: {e}"))
        })?;
    linker
        .func_wrap("aiway", "host_get_routing_url", host_get_routing_url)
        .map_err(|e| {
            crate::wasm::PluginError::LoadError(format!("register host_get_routing_url: {e}"))
        })?;
    linker
        .func_wrap(
            "aiway",
            "host_get_response_body_size",
            host_get_response_body_size,
        )
        .map_err(|e| {
            crate::wasm::PluginError::LoadError(format!(
                "register host_get_response_body_size: {e}"
            ))
        })?;
    linker
        .func_wrap(
            "aiway",
            "host_set_response_body_size",
            host_set_response_body_size,
        )
        .map_err(|e| {
            crate::wasm::PluginError::LoadError(format!(
                "register host_set_response_body_size: {e}"
            ))
        })?;
    linker
        .func_wrap("aiway", "host_log", host_log)
        .map_err(|e| crate::wasm::PluginError::LoadError(format!("register host_log: {e}")))?;
    linker
        .func_wrap_async(
            "aiway",
            "host_http_request",
            |caller: Caller<'_, WasmStoreCtx>,
             (req_ptr, req_len, resp_buf_ptr, resp_buf_len): (i32, i32, i32, i32)| {
                Box::new(host_http_request(
                    caller,
                    req_ptr,
                    req_len,
                    resp_buf_ptr,
                    resp_buf_len,
                )) as Box<dyn Future<Output = i32> + Send>
            },
        )
        .map_err(|e| {
            crate::wasm::PluginError::LoadError(format!("register host_http_request: {e}"))
        })?;

    #[cfg(feature = "model")]
    {
        linker
            .func_wrap("aiway", "host_get_model_name", host_get_model_name)
            .map_err(|e| {
                crate::wasm::PluginError::LoadError(format!("register host_get_model_name: {e}"))
            })?;
        linker
            .func_wrap("aiway", "host_get_model_provider", host_get_model_provider)
            .map_err(|e| {
                crate::wasm::PluginError::LoadError(format!(
                    "register host_get_model_provider: {e}"
                ))
            })?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// 宿主函数实现
// ---------------------------------------------------------------------------

/// 获取请求 ID，写入 WASM 缓冲区，返回数据实际长度（snprintf 语义）
fn host_request_id(mut caller: Caller<'_, WasmStoreCtx>, buf_ptr: i32, buf_len: i32) -> i32 {
    let id = http_ctx(&caller).request_id();
    write_to_wasm(&mut caller, buf_ptr, buf_len, id.as_bytes())
}

/// 获取请求时间戳（毫秒）
fn host_request_ts(caller: Caller<'_, WasmStoreCtx>) -> i64 {
    http_ctx(&caller).request_ts()
}

/// 是否为 SSE 连接
fn host_is_sse(caller: Caller<'_, WasmStoreCtx>) -> i32 {
    http_ctx(&caller).is_sse() as i32
}

/// 是否为 WebSocket 连接
fn host_is_websocket(caller: Caller<'_, WasmStoreCtx>) -> i32 {
    http_ctx(&caller).is_websocket() as i32
}

/// 获取路由名称，写入 WASM 缓冲区，返回数据实际长度（snprintf 语义）
fn host_get_route_name(mut caller: Caller<'_, WasmStoreCtx>, buf_ptr: i32, buf_len: i32) -> i32 {
    match http_ctx(&caller).get_route().map(|r| r.name.clone()) {
        Some(name) => write_to_wasm(&mut caller, buf_ptr, buf_len, name.as_bytes()),
        None => 0,
    }
}

/// 获取路由目标地址，写入 WASM 缓冲区，返回数据实际长度（snprintf 语义）
fn host_get_routing_url(mut caller: Caller<'_, WasmStoreCtx>, buf_ptr: i32, buf_len: i32) -> i32 {
    let url = http_ctx(&caller).get_routing_url().cloned();
    match url {
        Some(url) => write_to_wasm(&mut caller, buf_ptr, buf_len, url.as_bytes()),
        None => 0,
    }
}

/// 获取响应体大小，不存在时返回 -1
fn host_get_response_body_size(caller: Caller<'_, WasmStoreCtx>) -> i64 {
    http_ctx(&caller)
        .get_state::<i64>(HttpContext::RESPONSE_BODY_SIZE)
        .unwrap_or(-1)
}

/// 设置响应体大小
fn host_set_response_body_size(mut caller: Caller<'_, WasmStoreCtx>, size: i64) {
    http_ctx_mut(&mut caller).insert_state(HttpContext::RESPONSE_BODY_SIZE, size);
}

/// 获取模型名称，写入 WASM 缓冲区，返回数据实际长度（snprintf 语义）
#[cfg(feature = "model")]
fn host_get_model_name(mut caller: Caller<'_, WasmStoreCtx>, buf_ptr: i32, buf_len: i32) -> i32 {
    match http_ctx(&caller).get_proxy_model_name() {
        Some(name) => write_to_wasm(&mut caller, buf_ptr, buf_len, name.as_bytes()),
        None => 0,
    }
}

/// 获取模型提供商，bincode 序列化后写入 WASM 缓冲区，返回数据实际长度（snprintf 语义）
#[cfg(feature = "model")]
fn host_get_model_provider(
    mut caller: Caller<'_, WasmStoreCtx>,
    buf_ptr: i32,
    buf_len: i32,
) -> i32 {
    match http_ctx(&caller).get_proxy_model_provider() {
        Some(provider) => {
            let bytes = bincode::serialize(&provider).expect("serialize Provider failed");
            write_to_wasm(&mut caller, buf_ptr, buf_len, &bytes)
        }
        None => 0,
    }
}

/// 输出插件日志，自动追加 `[plugin_name]` 前缀
///
/// `level`: 日志级别（1=error, 2=warn, 3=info, 4=debug, 5=trace）
/// `msg_ptr`/`msg_len`: 日志消息在 WASM 线性内存中的位置
fn host_log(mut caller: Caller<'_, WasmStoreCtx>, level: i32, msg_ptr: i32, msg_len: i32) {
    let msg = read_from_wasm(&mut caller, msg_ptr, msg_len);
    let plugin_name = plugin_name(&caller);
    let formatted = format!("[{plugin_name}] {msg}");
    match level {
        LOG_ERROR => log::error!("{}", formatted),
        LOG_WARN => log::warn!("{}", formatted),
        LOG_INFO => log::info!("{}", formatted),
        LOG_DEBUG => log::debug!("{}", formatted),
        LOG_TRACE => log::trace!("{}", formatted),
        _ => log::info!("{}", formatted),
    }
}

// ---------------------------------------------------------------------------
// 工具函数
// ---------------------------------------------------------------------------

/// 从 Caller 中获取 HttpContext 不可变引用
///
/// # Safety
/// 指针在 `call_wasm` 期间始终有效，当前线程被阻塞或独占，不存在数据竞争。
fn http_ctx<'a>(caller: &'a Caller<'_, WasmStoreCtx>) -> &'a HttpContext {
    unsafe {
        (*caller.data().http_ctx.get())
            .expect("host function called without HttpContext set")
            .as_ref()
    }
}

/// 从 Caller 中获取 HttpContext 可变引用
///
/// # Safety
/// 同上，且调用期间无其他代码持有 HttpContext 的引用。
fn http_ctx_mut<'a>(caller: &'a mut Caller<'_, WasmStoreCtx>) -> &'a mut HttpContext {
    unsafe {
        (*caller.data().http_ctx.get())
            .expect("host function called without HttpContext set")
            .as_mut()
    }
}

/// 从 Caller 中获取插件名称
fn plugin_name<'a>(caller: &'a Caller<'_, WasmStoreCtx>) -> &'a str {
    unsafe {
        (*caller.data().plugin_name.get())
            .expect("host function called without plugin_name set")
            .as_ref()
    }
}

/// 从 WASM 线性内存读取字符串
fn read_from_wasm(caller: &mut Caller<'_, WasmStoreCtx>, ptr: i32, len: i32) -> String {
    let memory = caller
        .get_export("memory")
        .and_then(|e| e.into_memory())
        .expect("WASM module has no 'memory' export");
    let mut buf = vec![0u8; len as usize];
    memory
        .read(&mut *caller, ptr as usize, &mut buf)
        .expect("read from WASM memory failed");
    String::from_utf8_lossy(&buf).to_string()
}

/// 将数据写入 WASM 线性内存，返回数据实际长度（snprintf 语义）。
///
/// 当 `buf_len < data.len()` 时仅返回完整数据长度，
/// 调用方可通过比较返回值与缓冲区大小来检测截断并重试。
fn write_to_wasm(
    caller: &mut Caller<'_, WasmStoreCtx>,
    buf_ptr: i32,
    buf_len: i32,
    data: &[u8],
) -> i32 {
    let memory = caller
        .get_export("memory")
        .and_then(|e| e.into_memory())
        .expect("WASM module has no 'memory' export");
    if data.len() > buf_len as usize {
        return data.len() as i32;
    }
    memory
        .write(&mut *caller, buf_ptr as usize, data)
        .expect("write to WASM memory failed");
    data.len() as i32
}

/// 从 WASM 线性内存读取字节数组
fn read_bytes_from_wasm(caller: &mut Caller<'_, WasmStoreCtx>, ptr: i32, len: i32) -> Vec<u8> {
    let memory = caller
        .get_export("memory")
        .and_then(|e| e.into_memory())
        .expect("WASM module has no 'memory' export");
    let mut buf = vec![0u8; len as usize];
    memory
        .read(&mut *caller, ptr as usize, &mut buf)
        .expect("read from WASM memory failed");
    buf
}

/// 发起 HTTP 请求（异步）：反序列化请求 → 发送 → 序列化响应写回 WASM 内存
///
/// 通过 `func_wrap_async` 注册，`.await` 期间释放 worker 线程。
/// 返回 snprintf 语义长度（成功时 ≥0），负值表示错误码：
/// - -1: 请求反序列化失败
/// - -2: 不支持的 HTTP 方法
/// - -3: 请求执行失败
/// - -4: 响应体读取失败
async fn host_http_request(
    mut caller: Caller<'_, WasmStoreCtx>,
    req_ptr: i32,
    req_len: i32,
    resp_buf_ptr: i32,
    resp_buf_len: i32,
) -> i32 {
    // 1. 读取并反序列化请求
    let req_bytes = read_bytes_from_wasm(&mut caller, req_ptr, req_len);
    let request: HttpRequest = match bincode::deserialize(&req_bytes) {
        Ok(req) => req,
        Err(_) => return -1,
    };

    // 2. 构建 reqwest 请求
    let client = &NETWORK.client;
    let mut req_builder = match request.method.to_uppercase().as_str() {
        "GET" => client.get(&request.url),
        "POST" => client.post(&request.url),
        "PUT" => client.put(&request.url),
        "DELETE" => client.delete(&request.url),
        "PATCH" => client.patch(&request.url),
        "HEAD" => client.head(&request.url),
        _ => return -2,
    };

    for (key, value) in &request.headers {
        req_builder = req_builder.header(key.as_str(), value.as_str());
    }

    // 3. 设置请求体（优先级: multipart > form > body）
    if let Some(parts) = &request.multipart {
        let mut form = reqwest::multipart::Form::new();
        for part in parts {
            let mut form_part = reqwest::multipart::Part::bytes(part.value.clone());
            if let Some(ref file_name) = part.file_name {
                form_part = form_part.file_name(file_name.clone());
            }
            if let Some(ref mime_str) = part.mime_type
                && mime_str.parse::<mime::Mime>().is_ok()
            {
                form_part = form_part.mime_str(mime_str).unwrap();
            }
            form = form.part(part.key.clone(), form_part);
        }
        req_builder = req_builder.multipart(form);
    } else if let Some(form_map) = &request.form {
        req_builder = req_builder.form(form_map);
    } else if let Some(body) = &request.body {
        req_builder = req_builder.body(body.clone());
    }

    req_builder = req_builder.timeout(Duration::from_millis(request.timeout_ms));

    // 4. 执行请求（异步，.await 期间释放 worker 线程）
    let response = match req_builder.send().await {
        Ok(resp) => resp,
        Err(_) => return -3,
    };

    // 5. 读取响应
    let status = response.status().as_u16();
    let headers: Vec<(String, String)> = response
        .headers()
        .iter()
        .filter_map(|(k, v)| v.to_str().ok().map(|v| (k.to_string(), v.to_string())))
        .collect();

    let body = match response.bytes().await {
        Ok(b) => b.to_vec(),
        Err(_) => return -4,
    };

    // 6. 序列化响应并写回 WASM 内存
    let http_response = HttpResponse {
        status,
        headers,
        body,
    };
    let resp_bytes = bincode::serialize(&http_response).expect("serialize HttpResponse failed");

    write_to_wasm(&mut caller, resp_buf_ptr, resp_buf_len, &resp_bytes)
}
