#[macro_use]
extern crate rocket;

use rocket::Config;
use rocket::data::{ByteUnit, Limits};

mod config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    logging::init_log();

    start_http_server().await?;
    Ok(())
}

pub async fn start_http_server() -> anyhow::Result<()> {
    let mut builder = rocket::build().configure(Config {
        port: 5000,
        limits: Limits::default()
            .limit("json", ByteUnit::Mebibyte(5))
            .limit("data-form", ByteUnit::Mebibyte(100))
            .limit("file", ByteUnit::Mebibyte(100)),
        ..Config::debug_default()
    });

    builder = builder.mount("/", config::raft::api::routes());

    builder = builder.manage(config::new_raft_app(1).await);

    builder.launch().await?;

    Ok(())
}
