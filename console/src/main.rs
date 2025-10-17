use clap::Parser;

mod init;

mod args;
//mod config;
mod server;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[rocket::main]
async fn main() -> anyhow::Result<()> {
    let args = args::Args::parse();

    init::init(&args).await;

    server::start_http_server(&args).await?;
    Ok(())
}
