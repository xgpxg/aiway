use clap::Parser;

mod config;
mod fairing;
mod server;
mod openapi;
mod context;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// IP address, like 127.0.0.1
    #[arg(short, long, default_value = "127.0.0.1")]
    address: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 5000)]
    port: u16,
}

#[rocket::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    server::start_http_server(&args).await?;
    Ok(())
}
