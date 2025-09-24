use clap::Parser;

mod init;

mod config;
mod server;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// IP address
    #[arg(short, long, default_value = "config.yaml")]
    config: String,
}

#[rocket::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    init::init(&args).await;

    server::start_http_server().await?;
    Ok(())
}
