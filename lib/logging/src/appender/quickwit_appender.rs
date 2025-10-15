use crate::appender::LogEntry;
use reqwest::{Client, ClientBuilder};

pub struct QuickwitAppender {
    service: String,
    sender: tokio::sync::mpsc::UnboundedSender<LogEntry>,
}

impl QuickwitAppender {
    const MAX_BUFFER_SIZE: usize = 100;

    pub fn new<E: Into<String>, S: Into<String>>(endpoint: E, service: S) -> Self {
        let client = ClientBuilder::default()
            .connect_timeout(std::time::Duration::from_secs(3))
            .build()
            .unwrap();

        // 缓存，无界队列
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();

        let endpoint_clone = endpoint.into();
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
            sender,
            service: service.into(),
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
        let mut log_entry = LogEntry::from(buf.to_vec());
        log_entry.service = self.service.clone();
        self.sender.send(log_entry).ok();
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
