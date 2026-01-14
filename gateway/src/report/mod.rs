//! # 状态上报
//! ## 系统运行状态
//! 基础状态数据，如CPU、内存、磁盘使用情况，网络流量等。
//!
//! ## 计数器
//! 详情：[`protocol::gateway::state::Counter`]
//!

mod state;

use crate::Args;
use aiway_protocol::gateway::state::{NodeInfo, State};
pub use state::STATE;
use std::time::Duration;
use aiway_protocol::common::constants;

pub struct Reporter {
    client: reqwest::Client,
}

impl Reporter {
    pub fn new() -> Self {
        let client = reqwest::ClientBuilder::default()
            .connect_timeout(Duration::from_secs(
                constants::REPORT_STATE_INTERVAL,
            ))
            .build()
            .unwrap();
        Self { client }
    }

    async fn report(&self, addr: &str, state: &State) {
        match self.client.post(addr).json(state).send().await {
            Ok(resp) => {
                if !resp.status().is_success() {
                    log::error!("report failed, code: {}", resp.status());
                    // TODO 考虑写入到本地文件，按当前时间索引
                }
            }
            Err(e) => {
                log::error!("report failed: {}", e);
            }
        }
    }
}
pub fn init(args: &Args) {
    let console_addr = format!("http://{}/api/v1/gateway/report", args.console);
    let node_info = NodeInfo {
        node_id: args.node_id(),
        ip: args.address.clone(),
        port: args.port,
    };
    tokio::spawn(async move {
        // TODO 考虑15秒上报一次
        let reporter = Reporter::new();
        let mut timer = tokio::time::interval(Duration::from_secs(5));
        STATE.refresh(node_info.clone());
        loop {
            timer.tick().await;
            {
                let state = STATE.refresh(node_info.clone());
                reporter.report(&console_addr, &state).await;
            };
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_report() {
        let args = Args {
            address: "".to_string(),
            port: 0,
            console: "127.0.0.1:8080".to_string(),
            log_server: "127.0.0.1:7281".to_string(),
            cache_url: "".to_string(),
            cache_username: "".to_string(),
            cache_password: "".to_string(),
        };
        init(&args);
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}
