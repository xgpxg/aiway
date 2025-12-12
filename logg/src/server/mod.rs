mod index;

use crate::Args;
use common::dir::AppDir;
use rocket::Config;
use rocket::data::{ByteUnit, Limits};

pub async fn start_http_server(args: &Args) -> anyhow::Result<()> {
    let mut builder = rocket::build().configure(Config {
        port: args.port,
        limits: Limits::default()
            .limit("json", ByteUnit::Mebibyte(3))
            .limit("data-form", ByteUnit::Mebibyte(100))
            .limit("file", ByteUnit::Mebibyte(100)),
        log_level: rocket::config::LogLevel::Critical,
        cli_colors: false,
        ..Config::debug_default()
    });

    // aiway通用日志, Index: aiway-logs
    builder = builder.mount("/api/v1/aiway-logs", index::aiway_logs::routes());
    // 网关请求日志, Index: request-logs
    builder = builder.mount("/api/v1/request-logs", index::request_logs::routes());

    builder = builder.manage(index::aiway_logs::Logg::new(
        AppDir::log_dir()
            .join("index")
            .join("aiway")
            .to_str()
            .unwrap_or_default(),
    )?);
    builder = builder.manage(index::request_logs::Logg::new(
        AppDir::log_dir()
            .join("index")
            .join("request")
            .to_str()
            .unwrap_or_default(),
    )?);

    builder.launch().await?;

    Ok(())
}
