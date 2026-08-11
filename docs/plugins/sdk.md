# 插件 SDK

详情可参考：[插件 SDK](https://docs.rs/aiway-plugin/)

## Plugin trait

| 方法                   | 说明                                 |
|----------------------|------------------------------------|
| `name()`             | 插件名称（全局唯一）                         |
| `info()`             | 返回 `PluginInfo`（版本、默认配置、描述、README） |
| `on_request()`       | 请求阶段，可读写请求头、改写 URI                 |
| `on_request_body()`  | 请求体阶段，可读写请求体                       |
| `on_response()`      | 响应阶段，可读写响应头                        |
| `on_response_body()` | 响应体阶段，可读写响应体                       |
| `on_logging()`       | 日志阶段，请求结束后执行，不可中断流程                |

## PluginInfo

| 字段               | 类型               | 说明                |
|------------------|------------------|-------------------|
| `version`        | `Version`        | 插件版本（语义化版本）       |
| `default_config` | `Value`          | 默认配置              |
| `description`    | `String`         | 插件功能描述            |
| `readme`         | `Option<String>` | 使用手册（Markdown 文本） |

## 控制流 Outcome

各阶段返回 `PluginResult`（`Result<Outcome, PluginError>`）：

| 值                            | 含义                          |
|------------------------------|-----------------------------|
| `Outcome::Continue`          | 继续执行下一个插件或后续流程              |
| `Outcome::Respond(Response)` | 终止插件链，直接发送响应（如预检、缓存命中、mock） |

`Response` 结构：`status: u16`、`headers: Vec<(String, String)>`、`body: Vec<u8>`。

便捷方法：

- `Outcome::goon()`：继续执行
- `Outcome::respond(status, headers, body)`：主动响应
- `Outcome::reject(status, msg)`：拒绝请求，用于限流（429）、鉴权失败（403）、参数校验（400）等
- `Outcome::execute_error(msg)` / `Outcome::not_found(msg)`：返回对应错误

## PluginError

| 变体             | 说明           |
|----------------|--------------|
| `ExecuteError` | 插件业务逻辑错误     |
| `NotFound`     | 插件不存在        |
| `LoadError`    | 插件加载失败       |
| `SerdeError`   | 序列化/反序列化错误   |
| `HttpError`    | 发起 HTTP 调用错误 |

插件返回 `Err` 统一映射为网关 502。

## 导出宏

```rust
aiway_plugin::export_wasm!(MyPlugin);
```

生成 WASM 导出函数（`plugin_info`、`aiway_call` 等），插件必须调用。内部通过 `block_on` 执行异步钩子，WASM 环境无真正异步
I/O，插件中禁止使用异步网络等操作。

## 日志宏

提供格式化日志宏，输出到网关日志系统：

```rust
log_error!(ctx, "request failed: {}", err);
log_warn!(ctx, ...);
log_info!(ctx, ...);
log_debug!(ctx, ...);
log_trace!(ctx, ...);
```

## 插件上下文

`PluginContext` 是插件与网关交互的核心接口，宿主侧和 WASM 侧分别提供实现，插件开发者面向此 trait 编程，不依赖具体实现。

### 请求元数据

| 方法               | 返回               | 说明                            |
|------------------|------------------|-------------------------------|
| `request_id()`   | `String`         | 当前请求的唯一 ID                    |
| `request_ts()`   | `i64`            | 网关收到请求的时间戳（毫秒）                |
| `is_sse()`       | `bool`           | 是否为 SSE（Server-Sent Events）连接 |
| `is_websocket()` | `bool`           | 是否为 WebSocket 连接              |
| `method()`       | `Option<String>` | 请求方法                          |
| `uri()`          | `Option<Uri>`    | 请求 URI                        |

### 头部读写

| 方法                                    | 说明                               |
|---------------------------------------|----------------------------------|
| `get_request_header(name)`            | 读取原始请求头，跨阶段可用                    |
| `get_response_header(name)`           | 读取原始响应头，跨阶段可用                    |
| `set_request_header(name, value)`     | 覆盖写入请求头                          |
| `set_response_header(name, value)`    | 覆盖写入响应头                          |
| `append_request_header(name, value)`  | 多值追加请求头                          |
| `append_response_header(name, value)` | 多值追加响应头                          |
| `remove_request_header(name)`         | 移除请求头                            |
| `remove_response_header(name)`        | 移除响应头                            |
| `set_uri(uri)`                        | 改写请求 URI（路径改写，仅 on_request 阶段生效） |

### 请求/响应体

| 方法                        | 说明                               |
|---------------------------|----------------------------------|
| `request_body()`          | 读取当前请求体（仅 on_request_body 阶段有值）  |
| `set_request_body(body)`  | 覆盖请求体，后续插件与转发上游均可见               |
| `response_body()`         | 读取当前响应体（仅 on_response_body 阶段有值） |
| `set_response_body(body)` | 覆盖响应体                            |

### 插件配置

由网关在调用插件前注入，无需插件自行加载：

| 方法                 | 说明                            |
|--------------------|-------------------------------|
| `config()`         | 插件配置（JSON 字符串）                |
| `config_as_json()` | 将配置解析为 JSON `Value`（扩展 trait） |
| `config_as<T>()`   | 将配置反序列化为指定类型（扩展 trait）        |

### 路由信息

| 方法                  | 返回               | 说明                   |
|---------------------|------------------|----------------------|
| `get_route_name()`  | `Option<String>` | 匹配到的路由名称             |
| `get_routing_url()` | `Option<String>` | 负载均衡器选中的路由目标地址（含协议头） |

### 响应信息

| 方法                                        | 返回            | 说明                        |
|-------------------------------------------|---------------|---------------------------|
| `status()`                                | `Option<u16>` | 响应状态码（仅 on_response 阶段有值） |
| `get_response_body_size()`                | `Option<i64>` | 响应体大小（字节），未设置时返回 `None`   |
| `set_response_body_size(&mut self, size)` | -             | 设置响应体大小                   |

### 模型信息（仅模型插件可用）

以下方法仅在启用 `model` feature 时可用，仅在模型代理类型的插件中有效。

| 方法                     | 返回                 | 说明         |
|------------------------|--------------------|------------|
| `get_model_name()`     | `Option<String>`   | 请求使用的模型名称  |
| `get_model_provider()` | `Option<Provider>` | 命中的模型提供商信息 |

### 日志输出

`log(level, msg)` 为底层接口，`level` 使用日志级别常量；便捷方法输出到网关日志系统：

| 方法               | 级别    | 常量          |
|------------------|-------|-------------|
| `log_error(msg)` | ERROR | `LOG_ERROR` |
| `log_warn(msg)`  | WARN  | `LOG_WARN`  |
| `log_info(msg)`  | INFO  | `LOG_INFO`  |
| `log_debug(msg)` | DEBUG | `LOG_DEBUG` |
| `log_trace(msg)` | TRACE | `LOG_TRACE` |

### HTTP 请求

```rust
fn http_request(&self, req: &HttpRequest) -> Result<HttpResponse, PluginError>
```

发起出站 HTTP 请求，例如调用第三方 API、认证服务等。默认实现返回错误；WASM 插件通过宿主函数 `host_http_request` 委托网关发送，默认支持。

## 关联数据类型

### HttpRequest

| 字段           | 类型                                | 说明                                                         |
|--------------|-----------------------------------|------------------------------------------------------------|
| `method`     | `String`                          | HTTP 方法（GET、POST 等）                                        |
| `url`        | `String`                          | 请求 URL                                                     |
| `headers`    | `Vec<(String, String)>`           | 请求头列表                                                      |
| `body`       | `Option<Vec<u8>>`                 | 请求体（原始字节）                                                  |
| `form`       | `Option<HashMap<String, String>>` | URL 编码表单（与 body/multipart 互斥，优先级: multipart > form > body） |
| `multipart`  | `Option<Vec<FormPart>>`           | Multipart 表单（与 body/form 互斥，优先级最高）                         |
| `timeout_ms` | `u64`                             | 超时时间（毫秒），默认 10000                                          |

### HttpRequestBuilder

提供 Builder 模式便捷构造 `HttpRequest`：

```rust
let req = HttpRequestBuilder::new("POST", "https://api.example.com/verify")
.header("Authorization", "Bearer token")
.body(body_bytes)
.timeout_ms(5000)
.build();
```

支持的方法：`header()`、`body()`、`form()`、`add_form_field()`、`multipart()`、`add_multipart_part()`、`timeout_ms()`。

### FormPart（Multipart 表单字段）

| 字段          | 类型               | 说明                              |
|-------------|------------------|---------------------------------|
| `key`       | `String`         | 字段名                             |
| `value`     | `Vec<u8>`        | 字段值（文本或文件内容）                    |
| `file_name` | `Option<String>` | 文件名（文件上传时设置）                    |
| `mime_type` | `Option<String>` | MIME 类型（如 text/plain、image/png） |

### HttpResponse

| 字段        | 类型                      | 说明        |
|-----------|-------------------------|-----------|
| `status`  | `u16`                   | HTTP 状态码  |
| `headers` | `Vec<(String, String)>` | 响应头列表     |
| `body`    | `Vec<u8>`               | 响应体（原始字节） |

便捷方法：

- `text()` -> `Result<String>`：将响应体作为 UTF-8 文本返回
- `json<T>()` -> `Result<T>`：将响应体反序列化为指定类型
