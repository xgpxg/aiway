//! # 状态上报
//! ## 运行状态
//! 上报基础状态数据，如CPU、内存、磁盘使用情况，网络流量等。
//!
//! ## 监控指标
//! - 频繁鉴权失败
//!

mod state;

use std::time::Duration;

pub struct Reporter {
    interval: Duration,
}

impl Reporter {
    pub fn new(interval: Duration) -> Self {
        Self { interval }
    }

    pub fn run(&self) {
        loop {
            self.report();
            std::thread::sleep(self.interval);
        }
    }

    fn report(&self) {}
}
