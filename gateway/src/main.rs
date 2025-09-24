use clap::Parser;

mod constants;
mod context;
mod fairing;
mod init;
mod openapi;
mod report;
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

    /// Console address
    #[arg(short, long, default_value = "127.0.0.1:5001")]
    console: String,
}

#[rocket::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    init::init(&args).await;

    server::start_http_server(&args).await?;
    Ok(())
}
