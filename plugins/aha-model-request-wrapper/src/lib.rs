use plugin::{Plugin, PluginError, PluginInfo, Version, async_trait, export, serde_json};
use protocol::gateway::HttpContext;
use serde_json::Value;

/// # Aha模型请求参数转换
///
/// # 默认配置
/// 无
///
pub struct AhaModelRequestWrapperPlugin;

impl AhaModelRequestWrapperPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Plugin for AhaModelRequestWrapperPlugin {
    fn name(&self) -> &'static str {
        "AhaModelRequestWrapper"
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            version: Version::new(0, 1, 0),
            default_config: Default::default(),
            description: "Aha模型请求参数转换".to_string(),
        }
    }

    async fn execute(&self, context: &HttpContext, _config: &Value) -> Result<Value, PluginError> {
        let body = &serde_json::from_slice::<Value>(context.request.get_body().unwrap())
            .map_err(|e| PluginError::ExecuteError(e.to_string()))?;
        let model = body["model"].as_str().ok_or(PluginError::ExecuteError(
            "model field is not found".to_string(),
        ))?;
        let input = body["input"].as_str().ok_or(PluginError::ExecuteError(
            "input field is not found".to_string(),
        ))?;
        let voice = body["voice"].as_str().ok_or(PluginError::ExecuteError(
            "voice field is not found".to_string(),
        ))?;
        let voice_text = body["voice_text"]
            .as_str()
            .ok_or(PluginError::ExecuteError(
                "voice_text field is not found".to_string(),
            ))?;
        Ok(serde_json::json!(
            {
                "model": model,
                "messages": [
                    {
                        "role": "user",
                        "content": [
                            {
                                "type": "audio",
                                "audio_url": {
                                    "url": voice
                                }
                            },
                            {
                                "type": "text",
                                "text": input
                            }
                        ]
                    }
                ],
                "metadata": {
                    "prompt_text": voice_text
                }
            }
        ))
    }
}

// 导出插件
export!(AhaModelRequestWrapperPlugin);
