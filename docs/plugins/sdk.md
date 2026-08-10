# 开发一个插件

本文介绍如何开发一个 WASM 插件，并在网关中使用。

> 更详细的 API 参考，可查阅：[aiway-plugin](https://docs.rs/aiway-plugin)。

## 添加依赖

```toml
[dependencies]
aiway-plugin = "0.3"

[lib]
crate-type = ["cdylib"]
```

## 实现 Plugin trait

```rust
use aiway_plugin::{
    async_trait, log_info, serde_json::Value, Outcome, Plugin, PluginInfo, PluginResult,
    PluginContext, Version,
};

struct MyPlugin;

impl MyPlugin {
    pub fn new() -> Self {
        MyPlugin
    }
}

#[async_trait]
impl Plugin for MyPlugin {
    fn name(&self) -> &str {
        "my-plugin"
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            version: Version::new(0, 1, 0),
            default_config: Value::Null,
            description: "My first plugin".into(),
            readme: None,
        }
    }

    async fn on_request(&self, ctx: &mut dyn PluginContext) -> PluginResult {
        log_info!(ctx, "My First Plugin!");
        Ok(Outcome::Continue)
    }
}
```

## 导出插件

```rust
aiway_plugin::export_wasm!(MyPlugin);
```

## 编译

```bash
cargo build --release --target wasm32-wasip1

# 产物路径：target/wasm32-wasip1/release/my-plugin.wasm
```

## Plugin trait 接口

| 方法                 | 阶段    | 说明                                                      |
|--------------------|-------|---------------------------------------------------------|
| `on_request`       | 请求阶段  | 可改写请求头、请求 URI                                           |
| `on_request_body`  | 请求体阶段 | 通过 `ctx.request_body()` 读取、`set_request_body()` 改写请求体   |
| `on_response`      | 响应阶段  | 可改写响应头                                                  |
| `on_response_body` | 响应体阶段 | 通过 `ctx.response_body()` 读取、`set_response_body()` 改写响应体 |
| `on_logging`       | 日志阶段  | 请求结束后记录日志，无返回值                                          |

阶段方法返回 `PluginResult`（即 `Result<Outcome, PluginError>`），通过 `Outcome` 控制流程：

| 返回值                          | 行为                                |
|------------------------------|-----------------------------------|
| `Outcome::Continue`          | 继续执行下一个插件                         |
| `Outcome::Respond(Response)` | 插件直接响应，终止后续插件与转发（如鉴权失败、缓存命中、mock） |

常用便捷构造：

- `Outcome::goon()`：继续执行，等价于 `Ok(Outcome::Continue)`
- `Outcome::reject(status, msg)`：拒绝请求，常用于限流(429)、鉴权失败(403)、参数校验(400)
- `Outcome::respond(status, headers, body)`：自定义主动响应
- `Outcome::execute_error(msg)` / `Outcome::not_found(msg)`：返回业务错误

## 上下文 API

`PluginContext` 提供请求全生命周期的数据读写，插件开发者面向此 trait 编程：

| 分类   | 方法                                                                                                    |
|------|-------------------------------------------------------------------------------------------------------|
| 请求信息 | `request_id()` `request_ts()` `method()` `uri()` `set_uri()` `is_sse()` `is_websocket()`              |
| 路由信息 | `get_route_name()` `get_routing_url()`                                                                |
| 请求头  | `get_request_header()` `set_request_header()` `append_request_header()` `remove_request_header()`     |
| 响应头  | `get_response_header()` `set_response_header()` `append_response_header()` `remove_response_header()` |
| 请求体  | `request_body()` `set_request_body()`                                                                 |
| 响应体  | `response_body()` `set_response_body()`                                                               |
| 响应信息 | `status()` `get_response_body_size()` `set_response_body_size()`                                      |
| 模型信息 | `get_model_name()` `get_model_provider()`（仅启用 `model` feature 时可用）                                    |

### 配置获取

插件配置由控制台下发，经上下文获取：

```rust
use aiway_plugin::PluginContextExt; // config_as 等扩展方法

// 反序列化为自定义类型（推荐）
let cfg: MyConfig = ctx.config_as() ?;

// 原始 JSON
let json: Option<aiway_plugin::serde_json::Value> = ctx.config_as_json();
```

### 日志

使用格式化宏输出日志（对应 ERROR/WARN/INFO/DEBUG/TRACE 五个级别）：

```rust
log_info!(ctx, "request {} failed: {}", ctx.request_id(), err);
log_error!(ctx, "...");
log_warn!(ctx, "...");
log_debug!(ctx, "...");
log_trace!(ctx, "...");
```

### 发起 HTTP 调用

插件可通过 `http_request()` 调用外部服务，支持普通 body、URL 编码表单与 multipart：

```rust
use aiway_plugin::HttpRequestBuilder;

let resp = ctx.http_request(
& HttpRequestBuilder::new("POST", "https://api.example.com/submit")
.header("Content-Type", "application/json")
.body(r#"{"key":"value"}"#.into())
.timeout_ms(5_000)
.build(),
) ?;

let text = resp.text() ?;
let json: aiway_plugin::serde_json::Value = resp.json() ?;
```

## 上传到控制台

在控制台插件管理页面上传 `.wasm` 文件，配置插件参数后自动同步到网关节点。

至此，一个插件就开发好了。
