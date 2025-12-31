//! # 插件管理器
//! - 加载插件
//! - 列出可用插件
mod client;
mod plugins;

pub use plugins::PluginFactory;
use std::sync::OnceLock;

static CONSOLE: OnceLock<String> = OnceLock::new();
pub async fn init(console: &str) {
    CONSOLE.set(console.to_string()).unwrap();
    PluginFactory::init().await;
}
