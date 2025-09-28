use plugin::{Plugin, PluginError, export, async_trait};
use protocol::gateway::HttpContext;

// 示例插件
pub struct DemoPlugin;

impl DemoPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Plugin for DemoPlugin {
    fn name(&self) -> &'static str {
        "demo"
    }

    // 实现插件逻辑
    async fn execute(&self, context: &HttpContext, config: &serde_json::Value) -> Result<(), PluginError> {
        println!("run demo plugin, context: {:?}", context);
        println!("config: {:?}", config);
        Ok(())
    }
}

// 导出插件
export!(DemoPlugin);
