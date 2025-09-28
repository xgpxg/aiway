use crate::Args;
use crate::router::{ConfigFactory, PluginFactory, Router, Servicer};
use logging::LogAppender;

pub async fn init(_args: &Args) {
    // 初始化日志
    logging::init_log_with(
        LogAppender::all(),
        logging::Config {
            dir: Some("logs".to_string()),
            quickwit_endpoint: Some("127.0.0.1:7280".to_string()),
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
    pubsub::init("127.0.0.1:4222").await.unwrap();

    // 初始化网关配置
    ConfigFactory::init().await;

    // 初始化插件
    PluginFactory::init().await;

    // 初始化路由
    Router::init().await;

    // 初始化服务
    Servicer::init().await;
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
