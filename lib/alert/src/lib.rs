//! # 告警
//! 用于推送告警消息。
//!
//! - 告警消息不一定是警告或者错误，也可以是普通消息，便于统一处理和推送。
//! - 告警应该仅在必要时触发，避免频繁推送。
//! - 统一由控制台发送，网关节点推送告警消息到控制台即可，无需关注具体的发送逻辑。
//! - 控制台接收到告警消息后，会根据配置的推送渠道进行推送，并将消息详细内容保存下来，以便在控制台展示。
//!
//! 目前支持的推送渠道：
//! - 控制台（默认，不可关闭）
//! - 钉钉
//! - 企微
//! - 飞书
//! - 邮件
//! - 自定义接口
//!
//! ## 使用方式
//! ```rust
//! use alert::Alert;
//! Alert::info("标题", "内容");
//! Alert::warn("标题", "内容");
//! Alert::error("标题", "内容");
//! ```

pub mod pusher;

use aiway_protocol::gateway::alert::AlertMessage;
use std::sync::OnceLock;

#[derive(Debug)]
pub struct Alert {
    /// 控制台地址，格式：127.0.0.1:7000
    console: String,
    /// HTTP客户端
    client: reqwest::Client,
}

impl Alert {
    /// 控制台接收告警消息的接口
    const ALERT_API: &'static str = "/api/v1/gateway/alert";

    /// 创建一个Alert实例
    /// - console: 控制台地址，格式：IP:PORT，例如：127.0.0.1:7000
    pub fn new(console: String) -> Self {
        Alert {
            console: format!("http://{}{}", console, Self::ALERT_API),
            client: Default::default(),
        }
    }

    fn info_(&self, title: &str, content: &str) {
        let message = AlertMessage::info(title, content);
        self.send(message);
    }

    fn warn_(&self, title: &str, content: &str) {
        let message = AlertMessage::warn(title, content);
        self.send(message);
    }

    fn error_(&self, title: &str, content: &str) {
        let message = AlertMessage::error(title, content);
        self.send(message);
    }

    /// 发送告警消息到控制台
    fn send(&self, message: AlertMessage) {
        let client = self.client.clone();
        let console = self.console.clone();
        tokio::spawn(async move {
            client.post(&console).json(&message).send().await?;
            Ok::<(), reqwest::Error>(())
        });
    }

    /// 推送INFO级别的消息到控制台
    pub fn info(title: &str, content: &str) {
        let alert = ALERT.get().unwrap();
        alert.info_(title, content);
    }

    /// 推送发送WARN级别的消息到控制台
    pub fn warn(title: &str, content: &str) {
        let alert = ALERT.get().unwrap();
        alert.warn_(title, content);
    }

    /// 推送发送ERROR级别的消息到控制台
    pub fn error(title: &str, content: &str) {
        let alert = ALERT.get().unwrap();
        alert.error_(title, content);
    }
}

static ALERT: OnceLock<Alert> = OnceLock::new();

/// 初始化
///
/// - console: 控制台地址，格式：IP:PORT，例如：127.0.0.1:7000
pub fn init(console: String) {
    ALERT.set(Alert::new(console)).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_alert() {
        init("127.0.0.1:7000".to_string());
        Alert::info("测试标题", "测试内容");

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
