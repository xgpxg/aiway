#[macro_use]
extern crate rocket;

use anyhow::Context;
use clap::Parser;
use rocket::Config;
use rocket::data::{ByteUnit, Limits};
use std::fs;
use std::net::IpAddr;
use std::path::Path;
use std::str::FromStr;

mod app;
mod config;
mod event;
mod namespace;
mod raft;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, default_value = "127.0.0.1")]
    address: String,
    #[arg(short, long, default_value_t = 8000)]
    port: u16,
    #[arg(short, long, default_value = "./data")]
    data_dir: String,
    #[arg(short, long, default_value_t = 1)]
    node_id: u64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // 初始化日志
    logging::init_log();

    // 初始化目录
    init_dir(&args)?;

    // 初始化app
    app::init().await?;

    start_http_server(&args).await?;
    Ok(())
}

async fn start_http_server(args: &Args) -> anyhow::Result<()> {
    let mut builder = rocket::build().configure(Config {
        address: IpAddr::from_str(&args.address)?,
        port: args.port,
        limits: Limits::default()
            .limit("json", ByteUnit::Mebibyte(5))
            .limit("data-form", ByteUnit::Mebibyte(100))
            .limit("file", ByteUnit::Mebibyte(100)),
        ..Config::debug_default()
    });

    builder = builder.mount("/", raft::api::routes());
    builder = builder.mount("/config", config::server::api::routes());

    //builder = builder.manage(App::new(&args).await);

    builder.launch().await?;

    Ok(())
}

fn init_dir(args: &Args) -> anyhow::Result<()> {
    // 数据目录
    let data_dir = Path::new(&args.data_dir);
    fs::create_dir_all(data_dir).context("Failed to create data dir")?;

    // 数据库文件
    let db_file = data_dir.join("db").join("config.db");
    if !Path::exists(&db_file) {
        fs::create_dir_all(db_file.parent().unwrap())?;
        fs::File::create(db_file)?;
    }

    // raft 日志目录
    let raft_dir = data_dir.join("raft");
    if !Path::exists(&raft_dir) {
        fs::create_dir_all(raft_dir)?;
    }

    Ok(())
}
