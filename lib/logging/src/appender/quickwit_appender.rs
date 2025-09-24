use reqwest::{Client, ClientBuilder};
use serde::Serialize;

pub struct QuickwitAppender {
    client: Client,
    endpoint: String,
    sender: tokio::sync::mpsc::UnboundedSender<LogEntry>,
}

#[derive(Debug, Serialize)]
pub struct LogEntry {
    time: String,
    service: String,
    level: String,
    message: String,
}

/// 支持的格式：2025-01-01 00:00:00.000 INFO message
impl From<Vec<u8>> for LogEntry {
    fn from(value: Vec<u8>) -> Self {
        let value = String::from_utf8(value).unwrap();
        let time = &value[0..23];
        let level = &value[24..29].trim();
        let message = &value[30..].trim();
        Self {
            time: time.to_string(),
            service: "gateway".to_string(),
            level: level.to_string(),
            message: message.to_string(),
        }
    }
}

pub struct QuickwitLogReq {
    index_name: String,
    doc: String,
}

impl QuickwitAppender {
    const MAX_BUFFER_SIZE: usize = 100;

    pub fn new<E: Into<String>>(endpoint: E) -> Self {
        let client = ClientBuilder::default()
            .connect_timeout(std::time::Duration::from_secs(3))
            .build()
            .unwrap();

        // 缓存，无界队列
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();

        let endpoint = endpoint.into();
        let endpoint_clone = endpoint.clone();
        let client_clone = client.clone();
        tokio::spawn(async move {
            let mut buffer = Vec::new();
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

            loop {
                tokio::select! {
                    Some(log_entry) = receiver.recv() => {
                        buffer.push(log_entry);
                        if buffer.len() >= Self::MAX_BUFFER_SIZE {
                            Self::send_batch(&client_clone, &endpoint_clone, &buffer).await;
                            buffer.clear();
                        }
                    }
                    _ = interval.tick() => {
                        if !buffer.is_empty() {
                            Self::send_batch(&client_clone, &endpoint_clone, &buffer).await;
                            buffer.clear();
                        }
                    }
                }
            }
        });
        Self {
            client,
            endpoint,
            sender,
        }
    }

    async fn send_batch(client: &Client, endpoint: &str, logs: &[LogEntry]) {
        // 一行一个json
        let lines = logs
            .iter()
            .map(|log| serde_json::to_string(log).unwrap())
            .collect::<Vec<_>>()
            .join("\n");

        // 日志服务异常，不再输出日志，仅输出到stderr，防止重复发送日志。
        if let Err(e) = client.post(endpoint).body(lines).send().await {
            eprintln!("write log to http endpoint error: {}", e);
        }
    }
}

impl std::io::Write for QuickwitAppender {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let log_entry = LogEntry::from(buf.to_vec());
        self.sender.send(log_entry).ok();
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
