//! # 模型调用日志推送
//! 将模型调用日志推送到日志服务（quickwit/logg）。

use reqwest::ClientBuilder;
use std::sync::OnceLock;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
pub struct ModelCallLogging {
    sender: UnboundedSender<Vec<u8>>,
}

impl ModelCallLogging {
    const LOG_BUFFER_SIZE: usize = 1000;

    pub fn new<E: Into<String>>(endpoint: E) -> Self {
        let (sender, mut receiver) = mpsc::unbounded_channel::<Vec<u8>>();

        let client = ClientBuilder::default()
            .connect_timeout(std::time::Duration::from_secs(3))
            .build()
            .unwrap();

        let endpoint = endpoint.into();

        tokio::spawn(async move {
            let mut buffer = Vec::with_capacity(Self::LOG_BUFFER_SIZE);
            loop {
                tokio::select! {
                    Some(log) = receiver.recv() => {
                        buffer.push(log);
                        if buffer.len() >= Self::LOG_BUFFER_SIZE {
                            Self::flush_logs(&client, &endpoint, &mut buffer).await;
                        }
                    }
                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {
                        if !buffer.is_empty() {
                            Self::flush_logs(&client, &endpoint, &mut buffer).await;
                        }
                    }
                }
            }
        });

        Self { sender }
    }

    #[inline]
    pub fn log(&self, log: Vec<u8>) {
        if let Err(err) = self.sender.send(log) {
            log::error!("Failed to send model call log: {}", err);
        }
    }

    async fn flush_logs(client: &reqwest::Client, endpoint: &str, buffer: &mut Vec<Vec<u8>>) {
        if buffer.is_empty() {
            return;
        }
        let mut lines = Vec::new();
        for (i, log) in buffer.iter().enumerate() {
            if i > 0 {
                lines.extend_from_slice(b"\n");
            }
            lines.extend_from_slice(log);
        }
        let lines = String::from_utf8(lines).unwrap_or_default();

        if let Err(e) = client.post(endpoint).body(lines).send().await {
            eprintln!("write model call log to http endpoint error: {}", e);
        }

        buffer.clear();
    }
}

static MODEL_CALL_LOGGING: OnceLock<ModelCallLogging> = OnceLock::new();

pub(crate) fn init(endpoint: String) {
    MODEL_CALL_LOGGING
        .set(ModelCallLogging::new(endpoint))
        .expect("init model call logging failed");
}

/// 记录模型调用日志
pub fn log_model_call(log: Vec<u8>) {
    if let Some(logging) = MODEL_CALL_LOGGING.get() {
        logging.log(log);
    } else {
        log::error!("model call logging not initialized");
    }
}
