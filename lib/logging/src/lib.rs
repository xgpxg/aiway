//! # 日志
//! ## 输出到控制台
//! 默认std:io::stderr
//!
//! ## 输出到文件
//! 默认按天生成文件，异步写入。
//!
//! ## 输出到远程
//! 考虑使用http_writer，但是这个很慢，影响性能，待定。
//!

use crate::appender::QuickwitAppender;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::OnceLock;
use tracing_subscriber::fmt::time::ChronoLocal;
use tracing_subscriber::layer::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{Registry, fmt};

pub use log;
use tracing_appender::non_blocking::WorkerGuard;

mod appender;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct LogAppender: u32 {
        const CONSOLE = 1 << 0;  // 1
        const FILE    = 1 << 1;  // 2
        const QUICKWIT  = 1 << 2;  // 4
    }
}

#[derive(Debug)]
pub struct Config {
    pub dir: Option<String>,
    pub endpoint: Option<String>,
}

impl Config {
    fn default_dir() -> String {
        "logs".to_string()
    }
    fn default_endpoint() -> String {
        "http://127.0.0.1:7280/api/v1/gateway-logs/ingest".to_string()
    }
}
static LOG_GUARD: OnceLock<Vec<WorkerGuard>> = OnceLock::new();

pub fn init_log() {
    init_log_with(
        LogAppender::CONSOLE,
        Config {
            dir: Some("logs".to_string()),
            endpoint: Some(Config::default_endpoint()),
        },
    );
}
pub fn init_log_with(writer: LogAppender, config: Config) {
    let mut guards = vec![];

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        "info,rocket=warn,rocket::response::debug=error,rocket::launch=error".into()
    });

    let subscriber = Registry::default()
        .with(env_filter)
        // 输出到控制台
        .with(if writer.contains(LogAppender::CONSOLE) {
            let (appender, guard) = tracing_appender::non_blocking(std::io::stderr());
            let layer = fmt::Layer::default()
                .with_writer(appender)
                .compact()
                .with_level(true)
                .with_line_number(true)
                .with_timer(ChronoLocal::new("%Y-%m-%d %H:%M:%S.%3f".to_string()))
                .with_ansi(true);
            guards.push(guard);
            Some(layer)
        } else {
            None
        })
        // 输出到文件
        .with(if writer.contains(LogAppender::FILE) {
            let appender = tracing_appender::rolling::daily(
                config.dir.unwrap_or(Config::default_dir()),
                "app.log",
            );
            let (appender, guard) = tracing_appender::non_blocking(appender);
            let layer = fmt::Layer::default()
                .with_writer(appender)
                .compact()
                .with_level(true)
                .with_line_number(true)
                .with_timer(ChronoLocal::new("%Y-%m-%d %H:%M:%S.%3f".to_string()))
                .with_ansi(false);
            guards.push(guard);
            Some(layer)
        } else {
            None
        })
        // 输出到quickwit
        .with(if writer.contains(LogAppender::QUICKWIT) {
            if config.endpoint.is_none() {
                panic!("quickwit endpoint is required");
            }
            let quickwit_appender =
                QuickwitAppender::new(config.endpoint.unwrap_or(Config::default_endpoint()));
            let (appender, guard) = tracing_appender::non_blocking(quickwit_appender);

            let layer = fmt::Layer::default()
                .with_writer(appender)
                .compact()
                .with_level(true)
                .with_target(false)
                .with_line_number(false)
                .with_timer(ChronoLocal::new("%Y-%m-%d %H:%M:%S.%3f".to_string()))
                .with_ansi(false);
            guards.push(guard);
            Some(layer)
        } else {
            None
        });

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting global default subscriber failed");

    tracing_log::LogTracer::init().expect("failed to set logger");

    // 保持引用，non_blocking需要
    LOG_GUARD.get_or_init(|| guards);
}
