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

    async fn execute(&self, context: &HttpContext, config: &Value) -> Result<Value, PluginError> {
        Ok(serde_json::json!(
            {
                "model": "voxcpm1.5",
                "messages": [
                    {
                        "role": "user",
                        "content": [
                            {
                                "type": "audio",
                                "audio_url": {
                                    "url": "https://sis-sample-audio.obs.cn-north-1.myhuaweicloud.com/16k16bit.wav"
                                }
                            },
                            {
                                "type": "text",
                                "text": "VoxCPM is an innovative end-to-end TTS model from ModelBest, designed to generate highly realistic speech."
                            }
                        ]
                    }
                ],
                "metadata": {
                    "prompt_text": "华为致力于把数字世界带给每个人，每个家庭，每个组织，构建万物互联的智能世界。"
                }
            }
        ))
    }
}

// 导出插件
export!(AhaModelRequestWrapperPlugin);
