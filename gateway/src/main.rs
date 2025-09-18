use clap::Parser;

mod config;
mod constants;
mod context;
mod fairing;
mod init;
mod openapi;
mod router;
mod server;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// IP address, like 127.0.0.1
    #[arg(short, long, default_value = "127.0.0.1")]
    address: String,

    /// Port
    #[arg(short, long, default_value_t = 5000)]
    port: u16,
}

#[rocket::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    init::init(&args).await;

    server::start_http_server(&args).await?;
    Ok(())
}
