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
    cache::init_redis_cache(vec!["redis://127.0.0.1:6379"]).unwrap();
    #[cfg(feature = "standalone")]
    cache::init_share_cache().await.unwrap();

    // 初始化定时任务
    task::start().await.unwrap();
}

fn init_dir() -> anyhow::Result<()> {
    let data_dir = AppDir::data_dir();
    fs::create_dir_all(data_dir).context("Failed to create data directory")?;
    fs::create_dir_all(data_dir.join("sqlite")).context("Failed to create sqlite directory")?;
    fs::create_dir_all(data_dir.join("cache")).context("Failed to create cache directory")?;
    fs::create_dir_all(data_dir.join("temp")).context("Failed to create temp directory")?;

    let resources_dir = AppDir::resources_dir();
    fs::create_dir_all(resources_dir.join("web")).context("Failed to create web directory")?;

    Ok(())
}
