use crate::server::db::models::plugin::Plugin;
use rocket::serde::{Deserialize, Serialize};
use semver::Version;
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfoRes {
    /// 插件名，插件解析后获得
    pub name: String,
    /// 插件版本，插件解析后获得
    pub version: Version,
    /// 默认配置，插件解析后获得
    pub default_config: Value,
    /// 描述，插件解析后获得
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginListRes {
    #[serde(flatten)]
    pub inner: Plugin,
}
