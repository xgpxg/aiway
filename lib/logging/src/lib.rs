//! # 日志
//! ## 输出到控制台
//! 默认std:io::stderr
//!
//! ## 输出到文件
//! 默认按天生成文件，异步写入。
//!
//! ## 输出到远程
//! 需要部署quickwit
//!
//! ## 输出到本地索引
//! 仅单机模式启用，使用tantivy做索引
//!

#[cfg(feature = "local-storage")]
use crate::appender::LocalAppender;
use crate::appender::QuickwitAppender;
use std::fmt::Debug;
use std::sync::OnceLock;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::time::ChronoLocal;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{Registry, fmt};
mod appender;

pub use log;
#[cfg(feature = "local-storage")]
pub use tantivy;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct LogAppender: u32 {
        const CONSOLE = 1 << 0;  // 1
        const FILE    = 1 << 1;  // 2
        const QUICKWIT  = 1 << 2;  // 4
        const LOCAL = 1 << 3;  // 8
    }
}

#[derive(Debug)]
pub struct Config {
    /// 服务名，默认为当前进程名
    pub service: String,
    /// 日志目录，默认为程序启动目录下的logs，不存在则自动创建
    pub dir: Option<String>,
    /// quickwit服务地址，默认127.0.0.1:7280
    pub quickwit_endpoint: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dir: Some(Self::default_dir()),
            quickwit_endpoint: Some(Self::default_quickwit_endpoint()),
            service: current_process_name(),
        }
    }
}

impl Config {
    // 默认日志目录
    fn default_dir() -> String {
        "logs".to_string()
    }

    // 默认quickwit endpoint
    fn default_quickwit_endpoint() -> String {
        "127.0.0.1:7280".to_string()
    }

    // 日志索引ID
    const INDEX_ID: &'static str = "aiway-logs";

    // 构建quickwit restful的api
    fn build_quickwit_endpoint(&self) -> String {
        format!(
            "http://{}/api/v1/{}/ingest",
            self.quickwit_endpoint
                .clone()
                .unwrap_or(Self::default_quickwit_endpoint()),
            Self::INDEX_ID,
        )
    }
}

// 保持引用worker的引用
static HOLDING_WORKER_GUARDS: OnceLock<Vec<WorkerGuard>> = OnceLock::new();

/// 初始化日志
pub fn init_log() {
    init_log_with(LogAppender::CONSOLE, Config::default());
}

/// 使用配置初始化日志
pub fn init_log_with(writer: LogAppender, config: Config) {
    let mut guards = vec![];

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        "info,\
        rocket=warn,\
        rocket::response::debug=error,\
        rocket::launch=error,\
        rocket::server::_=error,\
        gateway::openapi::_=error\
        "
        .into()
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
                config.dir.clone().unwrap_or(Config::default_dir()),
                format!("{}.log", config.service),
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
            if config.quickwit_endpoint.is_none() {
                panic!("quickwit endpoint is required");
            }
            let quickwit_appender =
                QuickwitAppender::new(config.build_quickwit_endpoint(), config.service.clone());
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
    // 输出到本地，使用tantivy构建日志索引，仅单机模式使用
    #[cfg(feature = "local-storage")]
    let subscriber = subscriber.with(if writer.contains(LogAppender::LOCAL) {
        let local_appender = LocalAppender::new(config.dir.unwrap(), config.service);
        let (appender, guard) = tracing_appender::non_blocking(local_appender);

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
    HOLDING_WORKER_GUARDS.get_or_init(|| guards);
}

// 获取当前进程名
pub(crate) fn current_process_name() -> String {
    std::env::args()
        .next()
        .as_ref()
        .map(std::path::Path::new)
        .and_then(std::path::Path::file_name)
        .and_then(std::ffi::OsStr::to_str)
        .map(String::from)
        .unwrap_or_else(|| "unknown".to_string())
}
