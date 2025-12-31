use  plugin::serde_json::Value;
use plugin::{Plugin, PluginError, PluginInfo, Version, async_trait, export, serde_json};
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

    fn info(&self) -> PluginInfo {
        PluginInfo {
            version: Version::new(0, 1, 0),
            default_config: Default::default(),
            description: "Demo Plugin".to_string(),
        }
    }

    // 实现插件逻辑
    async fn execute(&self, _context: &HttpContext, _config: &Value) -> Result<Value, PluginError> {
        //println!("run demo plugin, context: {:?}", context);
        //println!("config: {:?}", config);
        Ok(Default::default())
    }
}

// 导出插件
export!(DemoPlugin);
