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
aiway-plugin = "0.3"
```

2. Implement Plugin

In `src/lib.rs`:

```rust
use aiway_plugin::{async_trait, log_info, Plugin, PluginError, PluginInfo, PluginContext, Version};
use http::request;
use serde_json::Value;

pub struct MyPlugin;

#[async_trait]
impl Plugin for MyPlugin {
    fn name(&self) -> &str {
        "my-plugin"
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            version: Version::new(0, 1, 0),
            default_config: Value::Null,
            description: "My First AIWay Plugin".to_string(),
            readme: None,
        }
    }

    async fn on_request(
        &self,
        _config: &Value,
        head: &mut request::Parts,
        ctx: &mut dyn PluginContext,
    ) -> Result<(), PluginError> {
        log_info!(ctx, "MyPlugin: received request {} {}", head.method, head.uri);

        // Modify request headers
        head.headers.insert("X-Custom-Header", "my-value".parse().unwrap());

        Ok(())
    }
}

// Export the plugin
aiway_plugin::export_wasm!(MyPlugin);
```

3. Build Plugin

```bash
cargo build --release --target wasm32-wasip1
```

## Available Hooks

| Trait Method       | When Called                                    |
|--------------------|------------------------------------------------|
| `on_request`       | HTTP request headers received, before proxying |
| `on_request_body`  | Request body received                          |
| `on_response`      | Response headers received from upstream        |
| `on_response_body` | Response body received (sync, not async)       |
| `on_logging`       | After request completes (logging only)         |

## Logging

Use the provided macros with context:

```rust
log_error!(ctx, "...");
log_warn!(ctx, "...");
log_info!(ctx, "...");
log_debug!(ctx, "...");
log_trace!(ctx, "...");
```

## HTTP Requests

Plugins can make outbound HTTP requests via `PluginContext::http_request`:

```rust
use aiway_plugin::{HttpRequestBuilder, FormPart};

// Simple GET
let resp = ctx.http_request(
& HttpRequestBuilder::new("GET", "https://api.example.com/data")
.header("Authorization", "Bearer token")
.build()
) ?;

let body_text = resp.text() ?;
let json: serde_json::Value = resp.json() ?;

// POST with JSON body
let resp = ctx.http_request(
& HttpRequestBuilder::new("POST", "https://api.example.com/submit")
.header("Content-Type", "application/json")
.body(r#"{"key":"value"}"#.into())
.timeout_ms(5_000)
.build()
) ?;

// Multipart form upload
let resp = ctx.http_request(
& HttpRequestBuilder::new("POST", "https://api.example.com/upload")
.add_multipart_part(FormPart {
key: "file".into(),
value: file_content,
file_name: Some("photo.png".into()),
mime_type: Some("image/png".into()),
})
.add_form_field("description", "A photo")
.build()
) ?;
```

## Context API

`PluginContext` provides request metadata:

```rust
let request_id = ctx.request_id();
let is_sse = ctx.is_sse();
let is_websocket = ctx.is_websocket();
let route_name = ctx.get_route_name();
let routing_url = ctx.get_routing_url();

// With `model` feature enabled:
let model_name = ctx.get_model_name();
let provider = ctx.get_model_provider();
```

