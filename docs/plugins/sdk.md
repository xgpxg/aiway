# 插件开发

插件编译为 WASM 模块后由网关加载执行。

## 依赖

```toml
[dependencies]
aiway-plugin = "0.3"
```

## Cargo.toml 配置

```toml
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
```

编译环境：
- 使用 `wasm32-wasip1` 目标
- 通过 Host API 与网关通信

## Plugin trait 接口

| 方法                 | 类型    | 说明    |
|--------------------|-------|-------|
| `on_request`       | async | 修改请求头 |
| `on_request_body`  | async | 修改请求体 |
| `on_response`      | async | 修改响应头 |
| `on_response_body` | sync  | 修改响应体 |
| `on_logging`       | async | 日志阶段  |

## 上传

在控制台插件管理页面上传 `.wasm` 文件，配置插件参数后自动同步到网关。

## 安全

- WASM 在沙箱中执行
- 无直接主进程内存访问
- 插件异常不影响网关稳定性
