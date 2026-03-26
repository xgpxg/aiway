# Quick Start

1. Create Plugin Project

```toml
[package]
name = "my-plugin"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
aiway-plugin = "0.2.0"
```

2. Implement Plugin

In `src/lib.rs`:

```rust
use aiway_plugin::{async_trait, Plugin, PluginInfo, Version, protocol::context::HttpContext};
use aiway_plugin::protocol::context::http::request;
use serde_json::Value;
use bytes::Bytes;

pub struct MyPlugin;

#[async_trait]
impl Plugin for MyPlugin {
    fn name(&self) -> &str {
        "my-plugin"
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            version: aiway_plugin::plugin_version!(),
            default_config: Value::Null,
            description: "My First AIWay Plugin".to_string(),
        }
    }

    async fn on_request(
        &self,
        config: &Value,
        head: &mut request::Parts,
        ctx: &mut HttpContext,
    ) -> Result<(), aiway_plugin::PluginError> {
        // Process the request here
        println!("MyPlugin: Received request {} {}", head.method, head.path);
        
        // Can modify request headers
        head.headers.insert("X-Custom-Header", "my-value".parse().unwrap());
        
        Ok(())
    }
}

// Export the plugin
aiway_plugin::export!(MyPlugin);
```

3. Build Plugin

```bash
cargo build --release
```

