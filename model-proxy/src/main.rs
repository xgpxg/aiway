mod server;
mod init;
mod components;
mod proxy;

use clap::Parser;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// IP address, like 127.0.0.1
    #[arg(short, long, default_value = "127.0.0.1")]
    address: String,

    /// Port
    #[arg(short, long, default_value_t = 7010)]
    port: u16,

    /// Console address
    #[arg(short, long, default_value = "127.0.0.1:7000")]
    console: String,

    /// 日志服务
    #[arg(short, long, default_value = "127.0.0.1:7280")]
    log_server: String,

}
#[rocket::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    init::init(&args).await;

    server::start_http_server(&args).await?;
    Ok(())
}
