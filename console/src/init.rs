use crate::config::config;
use crate::Args;
use logging::LogAppender;

pub async fn init(args: &Args) {
    // 初始化日志
    logging::init_log_with(
        LogAppender::all(),
        logging::Config {
            dir: Some("logs".to_string()),
            quickwit_endpoint: Some("127.0.0.1:7280".to_string()),
            ..Default::default()
        },
    );

    // 初始化配置
    config::init(args.config.as_str()).unwrap();

    // 初始化缓存
    cache::init_local_cache("cache/console").unwrap();
}
