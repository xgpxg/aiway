use plugin::{Plugin, PluginError, export};
use protocol::gateway::HttpContext;

// 示例插件
pub struct DemoPlugin;

impl DemoPlugin {
    pub fn new() -> Self {
        Self {}
    }
}
impl Plugin for DemoPlugin {
    fn name(&self) -> &'static str {
        "demo"
    }

    // 实现插件逻辑
    fn execute(&self, context: &HttpContext) -> Result<(), PluginError> {
        println!("run demo plugin, context: {:?}", context);
        Ok(())
    }
}

// 导出插件
export!(DemoPlugin);
