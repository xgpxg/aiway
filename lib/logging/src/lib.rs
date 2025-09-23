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
use crate::writer::HttpLogWriter;
use std::fmt::Debug;
use std::sync::OnceLock;
use tracing_subscriber::fmt::time::ChronoLocal;
use tracing_subscriber::layer::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{Registry, fmt};

pub use log;
mod writer;

static LOG_GUARD: OnceLock<tracing_appender::non_blocking::WorkerGuard> = OnceLock::new();

pub fn init_log() {
    let file_appender = tracing_appender::rolling::daily("logs", "app.log");
    let (file_appender, guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        "info,rocket=warn,rocket::response::debug=error,rocket::launch=error".into()
    });

    let subscriber = Registry::default()
        .with(env_filter)
        .with(
            fmt::Layer::default()
                .with_writer(std::io::stderr)
                .compact()
                .with_level(true)
                .with_line_number(true)
                .with_timer(ChronoLocal::new("%Y-%m-%d %H:%M:%S.%3f".to_string()))
                .with_ansi(true),
        )
        .with(
            fmt::Layer::default()
                .with_writer(file_appender)
                .compact()
                .with_level(true)
                .with_line_number(true)
                .with_timer(ChronoLocal::new("%Y-%m-%d %H:%M:%S.%3f".to_string()))
                .with_ansi(false),
        );

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting global default subscriber failed");

    tracing_log::LogTracer::init().expect("failed to set logger");

    // 保持引用，non_blocking需要
    LOG_GUARD.get_or_init(|| guard);
}

#[cfg(test)]
mod tests {
    use super::{HttpLogWriter, init_log};
    use tracing;

    #[tokio::test]
    async fn test_http_writer() {
        init_log();

        tracing::info!("hello world");
    }
}
