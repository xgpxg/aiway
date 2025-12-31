use crate::Args;
use crate::components::ModelFactory;
use alert::Alert;
use logging::{LogAppender, log};

pub async fn init(args: &Args) {
    // 初始化日志
    logging::init_log_with(
        LogAppender::CONSOLE | LogAppender::QUICKWIT,
        logging::Config {
            quickwit_endpoint: Some(args.log_server.clone()),
            ..Default::default()
        },
    );

    // 初始化缓存
    // #[cfg(feature = "cluster")]
    // cache::init_redis_cache(vec!["redis://127.0.0.1:6379"]).unwrap();
    // #[cfg(feature = "standalone")]
    // cache::init_share_cache().await.unwrap();

    // 初始化告警
    alert::init(args.console.clone());

    // 初始化插件管理器
    plugin_manager::init(&args.console).await;

    // 初始化模型
    ModelFactory::init().await;

    // 设置panic hook
    set_panic_hook();
}

fn set_panic_hook() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        log::error!("{}", info);

        Alert::error("网关节点出现异常，请关注", &info.to_string());

        hook(info);
    }));
}
