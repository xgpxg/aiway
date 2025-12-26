use clap::Parser;

mod components;
mod constants;
mod context;
mod fairing;
mod init;
mod openapi;
mod report;
mod server;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Listen address, like 127.0.0.1
    #[arg(short, long, default_value = "127.0.0.1")]
    address: String,

    /// Port
    #[arg(short, long, default_value_t = 7001)]
    port: u16,

    /// Console address
    #[arg(short, long, default_value = "127.0.0.1:7000")]
    console: String,

    /// Log server address
    #[arg(short, long, default_value = "127.0.0.1:7280")]
    log_server: String,

    /// Cache connection url
    #[arg(long, default_value = "redis://127.0.0.1:6379")]
    pub cache_url: String,

    /// Cache username
    #[arg(long, default_value = "")]
    pub cache_username: String,

    /// Cache password
    #[arg(long, default_value = "")]
    pub cache_password: String,
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
