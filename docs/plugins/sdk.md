# 开发一个插件

本文介绍如何开发一个插件，并在网关中使用。

> 更详细的API参考，可查阅：[aiway-plugin](https://docs.rs/aiway-plugin/0.3.3/aiway_plugin)。

## 添加依赖

```toml
[dependencies]
aiway-plugin = "0.3"

[lib]
crate-type = ["cdylib"]
```

## 实现 Plugin trait

```rust
use aiway_plugin::*;

struct MyPlugin;

#[async_trait]
impl Plugin for MyPlugin {
    fn name(&self) -> &str {
        "my-plugin"
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            version: Version::new(0, 1, 0),
            default_config: serde_json::json!({}),
            description: "My first plugin".into(),
        }
    }

    async fn on_request(
        &self,
        config: &Value,
        head: &mut request::Parts,
        ctx: &mut dyn PluginContext,
    ) -> Result<(), PluginError> {
        ctx.log_info("My First Plugin!");
        Ok(())
    }
}
```

## 导出插件

```rust
export_wasm!(MyPlugin);
```

## 编译

```bash
cargo build -r --target wasm32-wasip1

# 产物路径：target/wasm32-wasip1/release/my-plugin.wasm
```

## Plugin trait 接口

| 方法                 | 类型    | 说明    |
|--------------------|-------|-------|
| `on_request`       | async | 修改请求头 |
| `on_request_body`  | async | 修改请求体 |
| `on_response`      | async | 修改响应头 |
| `on_response_body` | sync  | 修改响应体 |
| `on_logging`       | async | 日志阶段  |

## 上传到控制台

在控制台插件管理页面上传 `.wasm` 文件，配置插件参数后自动同步到网关节点。

至此，一个插件就开发好了。