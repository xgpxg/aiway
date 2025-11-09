use chrono::Local;
use serde::{Deserialize, Serialize};

/// 告警消息
#[derive(Debug, Serialize, Deserialize)]
pub struct AlertMessage {
    /// 告警发生的时间，格式为：2022-01-01 00:00:00.000
    pub time: String,
    /// 告警级别
    pub level: AlertLevel,
    /// 告警标题
    pub title: String,
    /// 告警内容
    pub content: String,
}

/// 告警级别
#[derive(Debug, Serialize, Deserialize)]
pub enum AlertLevel {
    Info,
    Warn,
    Error,
}

/// 告警配置
///
/// 该配置由控制台维护
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// 钉钉配置
    pub dingding: DingdingConfig,
    /// 企业微信配置
    pub wecom: WecomConfig,
    /// 飞书配置
    pub feishu: FeishuConfig,
    /// 自定义WebHook
    pub custom: CustomConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuConfig {
    /// 是否启用
    pub enable: bool,
    /// 飞书机器人WebHook
    pub webhook: String,
    /// 触发关键词
    #[serde(default = "default_keyword")]
    pub keyword: String,
}

fn default_keyword() -> String {
    "aiway".to_string()
}

impl Default for FeishuConfig {
    fn default() -> Self {
        Self {
            enable: false,
            webhook: "".to_string(),
            keyword: "aiway".to_string(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DingdingConfig {
    /// 是否启用
    pub enable: bool,
    /// 钉钉机器人WebHook
    pub webhook: String,
    /// 触发关键词
    #[serde(default = "default_keyword")]
    pub keyword: String,
    /*/// @手机号
    pub at_mobiles: Vec<String>,
    /// 是否@所有人
    pub is_at_all: bool,*/
}
impl Default for DingdingConfig {
    fn default() -> Self {
        Self {
            enable: false,
            webhook: "".to_string(),
            keyword: "aiway".to_string(),
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WecomConfig {
    /// 是否启用
    pub enable: bool,
    /// 企业微信机器人WebHook
    pub webhook: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CustomConfig {
    /// 是否启用
    pub enable: bool,
    /// 地址
    pub webhook: String,
}

impl AlertMessage {
    pub fn info(title: &str, content: &str) -> Self {
        AlertMessage {
            time: Self::now(),
            level: AlertLevel::Info,
            title: title.into(),
            content: content.into(),
        }
    }
    pub fn warn(title: &str, content: &str) -> Self {
        AlertMessage {
            time: Self::now(),
            level: AlertLevel::Warn,
            title: title.into(),
            content: content.into(),
        }
    }
    pub fn error(title: &str, content: &str) -> Self {
        AlertMessage {
            time: Self::now(),
            level: AlertLevel::Error,
            title: title.into(),
            content: content.into(),
        }
    }

    #[inline]
    fn now() -> String {
        let local_time = Local::now();
        local_time.format("%Y-%m-%d %H:%M:%S.%3f").to_string()
    }
}
