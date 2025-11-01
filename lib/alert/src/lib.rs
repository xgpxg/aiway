//! # 告警
//! 用于推送告警消息。
//!
//! 目前支持的推送渠道：
//! - 控制台
//! - 钉钉
//! - 企微
//! - 飞书
//! - 自定义接口

use protocol::gateway::alert::{AlertConfig, AlertMessage};
use std::sync::Arc;

pub struct Alert {
    /// 告警配置
    config: Arc<AlertConfig>,
    client: reqwest::Client,
}

impl Alert {
    pub fn new(config: AlertConfig) -> Self {
        Alert {
            config: Arc::new(config),
            client: Default::default(),
        }
    }

    pub fn info(&self, title: &str, content: &str) {
        let message = AlertMessage::info(title, content);
        // TODO: send alert
    }
    pub fn warn(&self, title: &str, content: &str) {
        let message = AlertMessage::warn(title, content);
        // TODO: send alert
    }
    pub fn error(&self, title: &str, content: &str) {
        let message = AlertMessage::error(title, content);
        // TODO: send alert
    }

    fn send(&self, message: AlertMessage) -> Result<(), reqwest::Error> {
        let client = self.client.clone();
        let config = self.config.clone();
        tokio::spawn(async move {
            if config.console.enable {
                client
                    .post(&config.console.address)
                    .json(&message)
                    .send()
                    .await?;
            }
            if config.dingding.enable {
                client
                    .post(&config.dingding.address)
                    .json(&message)
                    .send()
                    .await?;
            }
            if config.wecom.enable {
                client
                    .post(&config.wecom.address)
                    .json(&message)
                    .send()
                    .await?;
            }
            if config.feishu.enable {
                client
                    .post(&config.feishu.address)
                    .json(&message)
                    .send()
                    .await?;
            }
            Ok::<(), reqwest::Error>(())
        });

        Ok(())
    }
}
