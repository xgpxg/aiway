//! # 日志服务
//! 基于tantivy实现，REST API兼容Quickwit
//!
use crate::server::start_http_server;
use clap::Parser;

mod server;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Server address
    #[arg(short, long, default_value_t = String::from("127.0.0.1"))]
    address: String,
    /// Server port
    #[arg(short, long, default_value_t = 7280)]
    port: u16,
}

#[rocket::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    start_http_server(&args).await?;
    Ok(())
}
