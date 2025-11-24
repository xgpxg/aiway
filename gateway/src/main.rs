use clap::Parser;

mod constants;
mod context;
mod fairing;
mod init;
mod openapi;
mod report;
mod components;
mod server;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// IP address, like 127.0.0.1
    #[arg(short, long, default_value = "127.0.0.1")]
    address: String,

    /// Port
    #[arg(short, long, default_value_t = 5000)]
    port: u16,

    /// Console address
    #[arg(short, long, default_value = "127.0.0.1:6000")]
    console: String,

    /// 日志服务
    #[arg(short, long, default_value = "127.0.0.1:7280")]
    log_server: String,
    // TODO 缓存服务
}

impl Args {
    pub fn node_id(&self) -> String {
        let digest = md5::compute(format!("{}:{}", self.address, self.port));
        format!("{:x}", digest)[..8].to_string()
    }
}

#[rocket::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    init::init(&args).await;

    server::start_http_server(&args).await?;
    Ok(())
}
