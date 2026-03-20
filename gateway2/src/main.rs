use clap::Parser;

mod components;
mod handler;
mod init;
mod report;
mod server;

mod gateway;
#[cfg(feature = "mcp-proxy")]
mod mcp_proxy;
mod service;
#[cfg(feature = "model-proxy")]
mod model_proxy;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Listen address, like 127.0.0.1
    #[arg(short, long, default_value = "127.0.0.1")]
    pub address: String,

    /// Port
    #[arg(short, long, default_value_t = 7001)]
    pub port: u16,

    /// Console address
    #[arg(short, long, default_value = "127.0.0.1:7000")]
    pub console: String,

    /// Log server address
    #[arg(short, long, default_value = "127.0.0.1:7280")]
    pub log_server: String,
}

impl Args {
    pub fn node_id(&self) -> String {
        let digest = md5::compute(format!("{}:{}", self.address, self.port));
        format!("{:x}", digest)[..8].to_string()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    init::init(&args).await;

    server::start_http_server(&args)?;

    // 接收ctl c
    tokio::signal::ctrl_c().await?;

    Ok(())
}
