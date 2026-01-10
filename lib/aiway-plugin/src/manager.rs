use crate::protocol::gateway::HttpContext;
use crate::{Plugin, PluginError};
use dashmap::DashMap;
use serde_json::Value;

pub struct PluginManager {
    plugins: DashMap<String, Box<dyn Plugin>>,
}
impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Default::default(),
        }
    }

    pub fn register(&self, plugin: Box<dyn Plugin>) {
        self.plugins.insert(plugin.name().to_string(), plugin);
    }

    pub async fn run(
        &self,
        name: &str,
        context: &HttpContext,
        config: &Value,
    ) -> Result<Value, PluginError> {
        if let Some(plugin) = self.plugins.get(name) {
            plugin.execute(context, config).await
        } else {
            Err(PluginError::NotFound(name.to_string()))
        }
    }

    pub fn clear(&self) {
        self.plugins.clear();
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
