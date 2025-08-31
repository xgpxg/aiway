#[macro_use]
extern crate rocket;

use clap::Parser;
use rocket::Config;
use rocket::data::{ByteUnit, Limits};
use std::net::IpAddr;
use std::str::FromStr;

mod config;

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

    start_http_server(args).await?;
    Ok(())
}

async fn start_http_server(args: Args) -> anyhow::Result<()> {
    let mut builder = rocket::build().configure(Config {
        address: IpAddr::from_str(&args.address)?,
        port: args.port,
        limits: Limits::default()
            .limit("json", ByteUnit::Mebibyte(5))
            .limit("data-form", ByteUnit::Mebibyte(100))
            .limit("file", ByteUnit::Mebibyte(100)),
        ..Config::debug_default()
    });

    builder = builder.mount("/", config::raft::api::routes());

    builder = builder.manage(config::new_raft_app(args.node_id).await);

    builder.launch().await?;

    Ok(())
}
