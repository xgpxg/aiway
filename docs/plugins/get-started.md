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

## 上传到控制台

在控制台插件管理页面上传 `.wasm` 文件，配置插件参数后自动同步到网关节点。

至此，一个插件就开发好了。


