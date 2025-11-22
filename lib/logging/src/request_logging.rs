//! # 网关请求日志推送
//! 用于将网关的请求日志推送到日志服务。
//! 注意该模块应该仅处理网关的请求日志。
//!
use reqwest::ClientBuilder;
use serde::{Deserialize, Serialize};
use std::cell::OnceCell;
use std::sync::OnceLock;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
pub struct RequestLogging {
    sender: UnboundedSender<Vec<u8>>,
}

impl RequestLogging {
    const LOG_BUFFER_SIZE: usize = 1000;
    pub fn new<E: Into<String>>(endpoint: E) -> Self {
        // 无界队列
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
                            Self::flush_logs(&client,&endpoint,&mut buffer).await;
                        }
                    }
                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {
                        if !buffer.is_empty() {
                            Self::flush_logs(&client,&endpoint,&mut buffer).await;
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
            log::error!("Failed to send request log: {}", err);
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

        // 日志服务异常，不再输出日志，仅输出到stderr，防止重复发送日志。
        if let Err(e) = client.post(endpoint).body(lines).send().await {
            eprintln!("write log to http endpoint error: {}", e);
        }

        buffer.clear();
    }
}

static REQUEST_LOGGING: OnceLock<RequestLogging> = OnceLock::new();

pub(crate) fn init(endpoint: String) {
    REQUEST_LOGGING
        .set(RequestLogging::new(endpoint))
        .expect("init request logging failed");
}

/// 记录请求日志
pub fn log_request(log: Vec<u8>) {
    if let Some(request_logging) = REQUEST_LOGGING.get() {
        request_logging.log(log);
    } else {
        log::error!("request logging not initialized");
    }
}
