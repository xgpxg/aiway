use protocol::gateway::alert::{
    AlertConfig, AlertMessage, DingdingConfig, FeishuConfig, WebhookConfig, WecomConfig,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, LazyLock};

////////////////////////////// 钉钉推送消息 ////////////////////////////////
#[derive(Debug, Serialize, Deserialize)]
struct DingDingMessage {
    #[serde(rename = "msgtype")]
    msg_type: String,
    markdown: DingDingMarkdownContent,
    at: Option<AtContent>,
}
#[derive(Debug, Serialize, Deserialize)]
struct DingDingMarkdownContent {
    title: String,
    text: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct AtContent {
    #[serde(default)]
    at_mobiles: Vec<String>,
    #[serde(default)]
    at_user_ids: Vec<String>,
    is_at_all: bool,
}

////////////////////////////// 企业微信推送消息 ////////////////////////////////
#[derive(Debug, Serialize, Deserialize)]
struct WecomMessage {
    #[serde(rename = "msgtype")]
    msg_type: String,
    markdown: WecomMarkdownContent,
}
#[derive(Debug, Serialize, Deserialize)]
struct WecomMarkdownContent {
    content: String,
}

////////////////////////////// 飞书推送消息 ////////////////////////////////
#[derive(Debug, Serialize, Deserialize)]
struct FeishuMessage {
    msg_type: String,
    content: FeishuContent,
}
#[derive(Debug, Serialize, Deserialize)]
struct FeishuContent {
    text: String,
}

/// 触发关键词，仅钉钉和飞书需要
const KW: &str = "aiway";

static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .pool_max_idle_per_host(10)
        .connect_timeout(std::time::Duration::from_secs(3))
        .read_timeout(std::time::Duration::from_secs(300))
        .build()
        .unwrap()
});

pub struct Pusher;
impl Pusher {
    async fn push_dingding(config: &DingdingConfig, message: &AlertMessage) {
        log::debug!("push message to dingding");

        let (title, content) = Self::format_title_and_content(&message.title, &message.content);

        let message = DingDingMessage {
            msg_type: "markdown".to_string(),
            markdown: DingDingMarkdownContent {
                title,
                text: content,
            },
            at: None,
        };
        match HTTP_CLIENT
            .post(config.webhook.as_str())
            .json(&message)
            .send()
            .await
        {
            Ok(response) => {
                log::debug!("dingding response: {}", response.text().await.unwrap());
            }
            Err(e) => {
                log::error!("send dingding message error: {}", e);
            }
        }
    }
    async fn push_wecom(config: &WecomConfig, message: &AlertMessage) {
        log::debug!("push message to wecom");

        let (_, content) = Self::format_title_and_content(&message.title, &message.content);

        let message = WecomMessage {
            msg_type: "markdown".to_string(),
            markdown: WecomMarkdownContent { content },
        };
        match HTTP_CLIENT
            .post(config.webhook.as_str())
            .json(&message)
            .send()
            .await
        {
            Ok(response) => {
                log::debug!("wecom response: {}", response.text().await.unwrap());
            }
            Err(e) => {
                log::error!("send wecom message error: {}", e);
            }
        }
    }

    async fn push_feishu(config: &FeishuConfig, message: &AlertMessage) {
        log::debug!("push message to feishu");

        let (_, content) = Self::format_title_and_content(&message.title, &message.content);

        let message = FeishuMessage {
            msg_type: "text".to_string(),
            content: FeishuContent { text: content },
        };
        match HTTP_CLIENT
            .post(config.webhook.as_str())
            .json(&message)
            .send()
            .await
        {
            Ok(response) => {
                log::debug!("feishu response: {}", response.text().await.unwrap());
            }
            Err(e) => {
                log::error!("send feishu message error: {}", e);
            }
        }
    }

    async fn push_webhook(config: &WebhookConfig, message: &AlertMessage) {
        todo!()
    }

    fn format_title_and_content(title: &str, content: &str) -> (String, String) {
        let title = format!("【{}】{}", KW, title);
        let content = format!("{title}\n\n{content}");
        (title, content)
    }

    pub fn push(config: Arc<AlertConfig>, message: AlertMessage) {
        let config = config.clone();
        tokio::spawn(async move {
            if config.dingding.enable {
                Pusher::push_dingding(&config.dingding, &message).await;
            }
            if config.wecom.enable {
                Pusher::push_wecom(&config.wecom, &message).await;
            }
            if config.feishu.enable {
                Pusher::push_feishu(&config.feishu, &message).await;
            }
            if config.webhook.enable {
                Pusher::push_webhook(&config.webhook, &message).await;
            }
            Ok::<(), reqwest::Error>(())
        });
    }
}
