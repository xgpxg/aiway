use serde::{Deserialize, Serialize};
use crate::gateway::ConfiguredPlugin;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Provider {
    /// 提供商名称，全局唯一
    pub name: String,
    /// 提供商 API 地址
    pub api_url: String,
    /// 提供商 API 密钥
    ///
    /// 如果是本地模型，可能不需要密钥，所以密钥是可选的
    pub api_key: Option<String>,
    /// 权重，默认为 1
    pub weight: u32,
    /// 目标模型名称
    ///
    /// 场景：对于同一个模型在不同提供商处的名字可能不一样，通过该模型名称映射提供商处的真实模型名称。
    /// 同时能够让系统以同一个模型名称对外提供服务，后续的模型升级不需要修改对外名称，
    /// 能够做到快速切换模型版本。
    pub target_model_name: Option<String>,
    /// 插件
    pub plugins: Option<ConfiguredPlugin>,
    /// Token 用量提取配置
    pub token_usage_config: Option<TokenUsageConfig>,
}

/// Token 用量提取配置，声明 JSON 路径从响应中提取 token 数量
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct TokenUsageConfig {
    /// prompt_tokens 的 JSON 路径，如 `usage.prompt_tokens`
    pub prompt_tokens: Option<String>,
    /// completion_tokens 的 JSON 路径，如 `usage.completion_tokens`
    pub completion_tokens: Option<String>,
    /// total_tokens 的 JSON 路径，如 `usage.total_tokens`
    pub total_tokens: Option<String>,
}
