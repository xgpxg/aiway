use crate::router::Plugins;
use crate::{Args, constants};
use conreg_client::conf::{
    ClientConfigBuilder, ConRegConfigBuilder, ConfigConfigBuilder, DiscoveryConfigBuilder,
};
use conreg_client::init_with;

pub async fn init(args: &Args) {
    // 初始化日志
    logging::init_log();

    // 初始化conreg
    init_client(args).await;

    // 初始化缓存
    cache::init_local_cache("cache").unwrap();

    // 初始化插件
    Plugins::init().await;
}

async fn init_client(args: &Args) {
    let config = ConRegConfigBuilder::default()
        .client(
            ClientConfigBuilder::default()
                .port(args.port)
                .build()
                .unwrap(),
        )
        .config(
            ConfigConfigBuilder::default()
                .server_addr("127.0.0.1:8000")
                .config_ids(vec![
                    constants::ROUTES_CONFIG_ID.to_string(),
                    constants::SERVICES_CONFIG_ID.to_string(),
                    constants::PLUGINS_CONFIG_ID.to_string(),
                ])
                .build()
                .unwrap(),
        )
        .discovery(
            DiscoveryConfigBuilder::default()
                .server_addr("127.0.0.1:8000")
                .build()
                .unwrap(),
        )
        .build()
        .unwrap();

    init_with(config).await;
}
