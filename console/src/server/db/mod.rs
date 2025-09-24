use crate::config::config;
use crate::config::config::AppConfig;
use anyhow::bail;
use rbatis::RBatis;
use std::sync::OnceLock;
use logging::log;

mod migrations;
pub mod models;
mod mysql;
mod sqlite;
pub mod tools;

static RB: OnceLock<RBatis> = OnceLock::new();

pub struct Pool;
impl Pool {
    pub fn get<'a>() -> anyhow::Result<&'a RBatis> {
        match RB.get() {
            None => {
                log::error!("rbatis not init");
                bail!("rbatis not init".to_string());
            }
            Some(rb) => Ok(rb),
        }
    }
}

pub async fn init() -> anyhow::Result<()> {
    let database = AppConfig::database();
    match database.url.as_str() {
        url if url.starts_with("sqlite") => sqlite::init(database).await,
        url if url.starts_with("mysql") => mysql::init(database).await,
        _ => bail!("database not support"),
    };

    // 单机模式下执行版本升级
    // 集群模式下需要提供升级脚本执行
    if AppConfig::mode() == &config::Mode::Standalone {
        migrations::run(&mut Pool::get()?.clone()).await;
    }

    Ok(())
}
