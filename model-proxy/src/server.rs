use crate::{Args, proxy};
use logging::log;
use rocket::data::{ByteUnit, Limits};
use rocket::fairing::AdHoc;
use rocket::{Config, routes};
use std::net::IpAddr;
use std::str::FromStr;

pub async fn start_http_server(args: &Args) -> anyhow::Result<()> {
    let mut builder = rocket::build().configure(Config {
        address: IpAddr::from_str(args.address.as_str())?,
        port: args.port,
        limits: Limits::default()
            .limit("json", ByteUnit::Mebibyte(5))
            .limit("data-form", ByteUnit::Mebibyte(100))
            .limit("file", ByteUnit::Mebibyte(100)),
        log_level: rocket::config::LogLevel::Off,
        cli_colors: false,
        ..Config::debug_default()
    });

    // OpenAI接口兼容
    builder = builder.mount("/", routes![proxy::api::chat_completions]);

    builder = builder.attach(AdHoc::on_liftoff("Print Banner", |_| {
        Box::pin(async {
            print_banner();
        })
    }));

    builder.launch().await?;

    Ok(())
}

fn print_banner() {
    use clap::Parser;
    let args = Args::parse();
    log::info!(
        "model-proxy started success, current version: {}, listening on: {}:{}",
        crate::VERSION,
        args.address,
        args.port
    );
}
