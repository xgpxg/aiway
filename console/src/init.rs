use crate::Args;
use crate::config::config;
use crate::server::db;
use anyhow::Context;
use common::dir::AppDir;
use common::id;
use logging::LogAppender;
use std::fs;

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

    // 初始化目录
    init_dir().unwrap();

    // 初始化id
    id::init();

    // 初始化数据库
    db::init().await.unwrap();

    // 初始化缓存
    cache::init_local_cache("cache/console").unwrap();
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
