//! # 插件管理器
//! - 加载插件
//! - 列出可用插件
mod client;
mod plugins;
pub mod wasm;

pub use plugins::PluginFactory;

pub use wasm::{
    AsyncTryInto, NetworkPlugin, Outcome, Plugin, PluginError, PluginInfo, Response, WasmPlugin,
    async_trait, plugin_from_bytes,
};
pub use wasm::{Bytes, Version, http, protocol, serde_json};

use std::sync::OnceLock;

static CONSOLE: OnceLock<String> = OnceLock::new();
pub async fn init(console: &str) {
    CONSOLE.set(console.to_string()).unwrap();
    PluginFactory::init().await;
}
