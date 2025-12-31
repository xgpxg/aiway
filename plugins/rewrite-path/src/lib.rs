use plugin::serde_json::json;
use plugin::{Plugin, PluginError, PluginInfo, Version, async_trait, export, serde_json};
use protocol::gateway::HttpContext;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

/// # 路径重写插件
///
/// 重写网关收到的路径，将重写后的路径转发到目标服务。
///
/// # 插件参数示例
/// ```json
/// {
///     "pattern": "/api/*",
///     "replacement": "/$1"
/// }
/// ```
///
pub struct RewritePathPlugin;

impl RewritePathPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewriteRule {
    /// 匹配模式（正则），例如：/api/*
    pub pattern: String,
    /// 替换字符串，如：/$1
    pub replacement: String,
}

static REGEX_CACHE: LazyLock<Mutex<HashMap<String, Regex>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[async_trait]
impl Plugin for RewritePathPlugin {
    fn name(&self) -> &'static str {
        "RewritePath"
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            version: Version::new(0, 1, 0),
            default_config: json!({
                "pattern": "/api/*",
                "replacement": "/$1"
            }),
            description: "Rewrite path plugin".to_string(),
        }
    }

    async fn execute(
        &self,
        context: &HttpContext,
        config: &serde_json::Value,
    ) -> Result<serde_json::Value, PluginError> {
        let rule: RewriteRule = serde_json::from_value(config.clone()).map_err(|e| {
            PluginError::ExecuteError(format!("Failed to parse rewrite rules: {}", e))
        })?;

        let path = context.request.get_path();

        let regex = {
            let mut cache = REGEX_CACHE.lock().unwrap();
            if let Some(cached_regex) = cache.get(&rule.pattern) {
                cached_regex.clone()
            } else {
                let new_regex = Regex::new(&rule.pattern).map_err(|e| {
                    PluginError::ExecuteError(format!("Failed to parse rewrite rules: {}", e))
                })?;
                cache.insert(rule.pattern.clone(), new_regex.clone());
                new_regex
            }
        };

        let rewritten_path = if regex.is_match(&path) {
            regex.replace_all(&path, &rule.replacement).to_string()
        } else {
            path
        };

        context.request.set_routing_path(rewritten_path);
        Ok(Default::default())
    }
}

// 导出插件
export!(RewritePathPlugin);
