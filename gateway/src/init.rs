use crate::report::STATE;
use crate::router::{ConfigFactory, Firewalld, PluginFactory, Router, Servicer};
use crate::{Args, report};
use alert::Alert;
use logging::LogAppender;

pub async fn init(args: &Args) {
    // 初始化日志
    logging::init_log_with(
        LogAppender::CONSOLE | LogAppender::QUICKWIT,
        logging::Config {
            quickwit_endpoint: Some(args.log_server.clone()),
            ..Default::default()
        },
    );

    // 初始化conreg
    //init_client(args).await;

    // 初始化缓存
    #[cfg(feature = "cluster")]
    cache::init_redis_cache(vec!["redis://127.0.0.1:6379"]).unwrap();
    #[cfg(feature = "standalone")]
    cache::init_share_cache().await.unwrap();

    // 初始化发布订阅
    //pubsub::init("127.0.0.1:4222").await.unwrap();

    // 初始化网关配置
    ConfigFactory::init().await;

    // 初始化插件
    PluginFactory::init().await;

    // 初始化路由
    Router::init().await;

    // 初始化服务
    Servicer::init().await;

    // 初始化防火墙
    Firewalld::init().await;

    // 初始化监控
    report::init(args);

    // 初始化告警
    alert::init(args.console.clone());

    // 设置panic hook
    set_panic_hook();
}

fn set_panic_hook() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        log::error!("{}", info);

        STATE.inc_status_request_count(500, 1);
        STATE.inc_http_connect_count(-1);

        Alert::error("网关节点出现异常，请关注", &info.to_string());

        hook(info);
    }));
}

// async fn init_client(args: &Args) {
//     let config = ConRegConfigBuilder::default()
//         .client(
//             ClientConfigBuilder::default()
//                 .port(args.port)
//                 .build()
//                 .unwrap(),
//         )
//         .config(
//             ConfigConfigBuilder::default()
//                 .server_addr("127.0.0.1:8000")
//                 .config_ids(vec![
//                     constants::ROUTES_CONFIG_ID.to_string(),
//                     constants::SERVICES_CONFIG_ID.to_string(),
//                     constants::PLUGINS_CONFIG_ID.to_string(),
//                 ])
//                 .build()
//                 .unwrap(),
//         )
//         .discovery(
//             DiscoveryConfigBuilder::default()
//                 .server_addr("127.0.0.1:8000")
//                 .build()
//                 .unwrap(),
//         )
//         .build()
//         .unwrap();
//
//     init_with(config).await;
// }
