mod client;
mod plugins;

pub use plugins::PluginFactory;
use std::sync::OnceLock;

static CONSOLE: OnceLock<String> = OnceLock::new();
pub async fn init(console: &str) {
    CONSOLE.set(console.to_string()).unwrap();
    PluginFactory::init().await;
}
