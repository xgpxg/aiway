use crate::protocol::context::HttpContext;
use crate::{Plugin, PluginError};
use dashmap::DashMap;
use serde_json::Value;
use aiway_protocol::context::{ResponseHeader, Session};

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

    pub async fn run_on_request(
        &self,
        name: &str,
        session: &mut Session,
        ctx: &mut HttpContext,
        config: &Value,
    ) -> Result<(), PluginError> {
        if let Some(plugin) = self.plugins.get(name) {
            plugin.on_request(session, ctx, config).await
        } else {
            Err(PluginError::NotFound(name.to_string()))
        }
    }

    pub async fn run_on_response(
        &self,
        name: &str,
        session: &mut Session,
        response: &mut ResponseHeader,
        ctx: &mut HttpContext,
        config: &Value,
    ) -> Result<(), PluginError> {
        if let Some(plugin) = self.plugins.get(name) {
            plugin.on_response(session,response, ctx, config).await
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
