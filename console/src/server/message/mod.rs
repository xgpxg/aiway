//! # 系统消息
//! 该模块用于系统异常时记录和推送异常消息。
//!
//! 消息类型分为系统通知和预警消息。
//! - 系统通知：系统内部通知，默认方式
//! - 预警消息：系统内部通知，邮件、短信、钉钉、企微、飞书等机器人
//!
//! 需要预警的场景：
//! - 收到Warn和Error级别的日志
//! - 主动调用预警
//! - 网关节点失联
//! - 其他必要系统组件失联
//!
//! 预警方式：
//! - 系统内部通知（默认）
//! - 邮件
//! - 短信
//! - 钉钉、企微、飞树等机器人
use chrono::{DateTime, Utc};

/// 消息定义
///
pub struct Message {
    /// 时间戳
    ts: DateTime<Utc>,
    /// 级别
    level: Level,
    /// 内容
    content: String,
}

pub enum Level {
    Info,
    Warn,
    Error,
}

pub struct Alert {}

impl Alert {
    pub fn alert(&self, message: Message) {
        println!("{} {}", message.ts, message.content);
    }
}
