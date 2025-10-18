//! # 日志服务
//! 基于tantivy实现，REST API兼容Quickwit
//!
use crate::server::start_http_server;
use clap::Parser;

mod server;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// 配置文件，YAML格式
    #[arg(short, long, default_value_t = 7281)]
    port: u16,
}

#[rocket::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    start_http_server(&args).await?;
    Ok(())
}
