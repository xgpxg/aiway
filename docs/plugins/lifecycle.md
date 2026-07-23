# 插件生命周期

插件可以通过”插件管理“页面添加或删除，网关会定期同步已添加的插件。

每个插件都是独立的，在系统运行期间，同一个插件仅存在一个实例，插件可以被复用。

当插件被删除时，插件实例随之释放，此时原插件将不可用。

插件通常情况下是无状态的，即插件内部不存储数据，可以随时修改和更换插件。

## 插件执行时机

- 请求阶段
  客户端请求 → 路由匹配 → 全局插件 → 鉴权 → 路由插件 → 请求服务

- 响应阶段
  服务响应 → 路由插件 → 全局插件 → 响应客户端

## 热更新

插件支持热更新，即插件版本升级后，网关会自动重新同步插件，无需重启。

## 插件上下文

插件上下文 `PluginContext` 是插件与网关交互的核心接口，定义了插件可访问的所有操作。
宿主侧和 WASM 插件侧分别提供实现，插件开发者面向此 trait 编程，不依赖具体实现。

### 请求元数据

| 方法               | 返回       | 说明                            |
|------------------|----------|-------------------------------|
| `request_id()`   | `String` | 当前请求的唯一 ID                    |
| `request_ts()`   | `i64`    | 网关收到请求的时间戳（毫秒）                |
| `is_sse()`       | `bool`   | 是否为 SSE（Server-Sent Events）连接 |
| `is_websocket()` | `bool`   | 是否为 WebSocket 连接              |

### 路由信息

| 方法                  | 返回               | 说明                   |
|---------------------|------------------|----------------------|
| `get_route_name()`  | `Option<String>` | 匹配到的路由名称             |
| `get_routing_url()` | `Option<String>` | 负载均衡器选中的路由目标地址（含协议头） |

### 响应信息

| 方法                                        | 返回            | 说明                      |
|-------------------------------------------|---------------|-------------------------|
| `get_response_body_size()`                | `Option<i64>` | 响应体大小（字节），未设置时返回 `None` |
| `set_response_body_size(&mut self, size)` | -             | 设置响应体大小                 |

### 模型信息（仅模型插件可用）

**注意**：以下两个方法仅在启用 `model` feature 时可用，仅在模型代理类型的插件中有效。

| 方法                     | 返回                 | 说明         |
|------------------------|--------------------|------------|
| `get_model_name()`     | `Option<String>`   | 请求使用的模型名称  |
| `get_model_provider()` | `Option<Provider>` | 命中的模型提供商信息 |

### 日志输出

`PluginContext` 提供多级别日志方法，日志将输出到网关的日志系统：

| 方法               | 级别    |
|------------------|-------|
| `log_error(msg)` | ERROR |
| `log_warn(msg)`  | WARN  |
| `log_info(msg)`  | INFO  |
| `log_debug(msg)` | DEBUG |
| `log_trace(msg)` | TRACE |


### HTTP 请求

```rust
fn http_request(&self, req: &HttpRequest) -> Result<HttpResponse, PluginError>
```

插件可以通过此方法发起出站 HTTP 请求，例如调用第三方 API、认证服务等。

- **宿主侧**：通过底层 HTTP 客户端直接发送
- **WASM 侧**：通过宿主函数 `host_http_request` 委托给网关处理

WASM 插件默认支持此功能，宿主侧插件需确保运行环境支持。

### 关联数据类型

#### HttpRequest

| 字段           | 类型                                | 说明                                 |
|--------------|-----------------------------------|------------------------------------|
| `method`     | `String`                          | HTTP 方法（GET、POST 等）                |
| `url`        | `String`                          | 请求 URL                             |
| `headers`    | `Vec<(String, String)>`           | 请求头列表                              |
| `body`       | `Option<Vec<u8>>`                 | 请求体（原始字节）                          |
| `form`       | `Option<HashMap<String, String>>` | URL 编码表单（与 body/multipart 互斥）      |
| `multipart`  | `Option<Vec<FormPart>>`           | Multipart 表单（与 body/form 互斥，优先级最高） |
| `timeout_ms` | `u64`                             | 超时时间（毫秒），默认 10000                  |

#### HttpRequestBuilder

提供 Builder 模式便捷构造 `HttpRequest`：

```rust
let req = HttpRequestBuilder::new("POST", "https://api.example.com/verify")
.header("Authorization", "Bearer token")
.body(body_bytes)
.timeout_ms(5000)
.build();
```

支持的方法：`header()`、`body()`、`form()`、`add_form_field()`、`multipart()`、`add_multipart_part()`、`timeout_ms()`。

#### FormPart（Multipart 表单字段）

| 字段          | 类型               | 说明                              |
|-------------|------------------|---------------------------------|
| `key`       | `String`         | 字段名                             |
| `value`     | `Vec<u8>`        | 字段值（文本或文件内容）                    |
| `file_name` | `Option<String>` | 文件名（文件上传时设置）                    |
| `mime_type` | `Option<String>` | MIME 类型（如 text/plain、image/png） |

#### HttpResponse

| 字段        | 类型                      | 说明        |
|-----------|-------------------------|-----------|
| `status`  | `u16`                   | HTTP 状态码  |
| `headers` | `Vec<(String, String)>` | 响应头列表     |
| `body`    | `Vec<u8>`               | 响应体（原始字节） |

便捷方法：

- `text()` -> `Result<String>`：将响应体作为 UTF-8 文本返回
- `json<T>()` -> `Result<T>`：将响应体反序列化为指定类型

### 类型擦除

`as_any_mut()` 方法供宿主侧内部通过 downcast 获取具体的 `HttpContext` 实例，插件开发者无需关注。