# SDK 使用

插件 SDK (`aiway-plugin`) 提供 Plugin trait 和工具函数。

## 依赖

```toml
[dependencies]
aiway-plugin = "0.3"
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
            version: Version::new(1, 0, 0),
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
        Ok(())
    }
}
```

## 导出插件

```rust
export_wasm!(MyPlugin);  // WASM 模式下导出
```

## Plugin trait 接口

| 方法 | 类型 | 说明 |
|------|------|------|
| `on_request` | async | 修改请求头 |
| `on_request_body` | async | 修改请求体 |
| `on_response` | async | 修改响应头 |
| `on_response_body` | sync | 修改响应体 |
| `on_logging` | async | 日志阶段 |
