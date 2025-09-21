use crate::{Plugin, PluginError};
use dashmap::DashMap;
use protocol::gateway::HttpContext;

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

    pub async fn run(&self, name: &str, context: &HttpContext) -> Result<(), PluginError> {
        if let Some(plugin) = self.plugins.get(name) {
            plugin.execute(context).await
        } else {
            Err(PluginError::NotFound(name.to_string()))
        }
    }

    pub fn clear(&self) {
        self.plugins.clear();
    }
}
