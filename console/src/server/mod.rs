use crate::args::Args;
use clap::Parser;
use rocket::Config;
use rocket::data::{ByteUnit, Limits};
use std::net::IpAddr;
use std::str::FromStr;
mod auth;
mod cert;
mod common;
pub mod db;
mod domain;
mod file;
mod firewall;
mod gateway;
mod key;
mod log;
mod mcp;
mod mcp_proxy;
mod message;
mod metrics;
mod model;
mod model_proxy;
mod node;
mod plugin;
mod route;
mod service;
mod system;
pub mod task;
mod user;
#[cfg(not(debug_assertions))]
mod web;

pub async fn start_http_server(args: &Args) -> anyhow::Result<()> {
    //let config = &AppConfig::server();
    let mut builder = rocket::build().configure(Config {
        address: IpAddr::from_str(args.address.as_str())?,
        port: args.port,
        limits: Limits::default()
            .limit("json", ByteUnit::Mebibyte(3))
            .limit("data-form", ByteUnit::Mebibyte(100))
            .limit("file", ByteUnit::Mebibyte(100)),
        log_level: rocket::config::LogLevel::Critical,
        cli_colors: false,
        ..Config::debug_default()
    });

    // 网关调用
    builder = builder.mount("/api/v1", gateway::api::routes());
    builder = builder.mount("/api/v1", model_proxy::api::routes());
    builder = builder.mount("/api/v1", mcp_proxy::api::routes());

    // 业务调用
    builder = builder.mount("/api/user", user::api::routes());
    builder = builder.mount("/api/route", route::api::routes());
    builder = builder.mount("/api/service", service::api::routes());
    builder = builder.mount("/api/key", key::api::routes());
    builder = builder.mount("/api/plugin", plugin::api::routes());
    builder = builder.mount("/api/metrics", metrics::api::routes());
    builder = builder.mount("/api/log", log::api::routes());
    builder = builder.mount("/api/cert", cert::api::routes());
    builder = builder.mount("/api/domain", domain::api::routes());
    builder = builder.mount("/api/firewall", firewall::api::routes());
    builder = builder.mount("/api/system", system::routes());
    builder = builder.mount("/api/message", message::api::routes());
    builder = builder.mount("/api/node", node::api::routes());
    builder = builder.mount("/api/model", model::api::routes());
    builder = builder.mount("/api/mcp", mcp::api::routes());

    builder = builder.mount("/file/", file::api::routes());

    builder = builder.manage(Args::parse());

    #[cfg(not(debug_assertions))]
    {
        builder = builder.mount("/", rocket::routes![web::web]);
    }

    builder.launch().await?;

    Ok(())
}
