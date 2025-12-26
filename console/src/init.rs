//use crate::config::config;
//use crate::config::config::AppConfig;
use crate::args::Args;
use crate::server::{db, task};
use anyhow::Context;
use common::dir::AppDir;
use common::id;
use logging::LogAppender;
use std::fs;

pub async fn init(args: &Args) {
    // 初始化日志
    logging::init_log_with(
        LogAppender::CONSOLE | LogAppender::QUICKWIT,
        logging::Config {
            quickwit_endpoint: Some(args.log_server.clone()),
            ..Default::default()
        },
    );

    // 初始化配置
    //config::init(args.config.as_str()).unwrap();

    // 初始化目录
    init_dir().unwrap();

    // 初始化id
    id::init();

    // 初始化数据库
    db::init(args).await.unwrap();

    // 初始化缓存
    #[cfg(feature = "cluster")]
    cache::init_redis_cache(args.cache_url.split(",").collect::<Vec<_>>()).unwrap();
    #[cfg(feature = "standalone")]
    cache::init_share_cache().await.unwrap();

    // 初始化定时任务
    task::start().await.unwrap();

    // 初始化告警
    alert::init(format!("{}:{}", args.address, args.port));
}

fn init_dir() -> anyhow::Result<()> {
    AppDir::init_all();

    let data_dir = AppDir::data_dir();
    let resources_dir = AppDir::resources_dir();

    fs::create_dir_all(data_dir.join("sqlite")).context("Failed to create sqlite directory")?;
    fs::create_dir_all(resources_dir.join("web")).context("Failed to create web directory")?;

    Ok(())
}
