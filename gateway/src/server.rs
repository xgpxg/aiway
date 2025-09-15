//! # 网关服务端
//!
//! ## 基本准则
//! - 精简实现，不要耦合任何复杂的业务逻辑，具体逻辑应由插件实现
//! - 插件化，通过可序列化的数据进行通信，同一类型的插件接口参数应保持一致
//! - 涉及到网络连接的，需池化、复用，避免频繁创建、销毁连接
//! - 网关应不依赖任何中间件，可水平扩展，每个节点需独立运行，无相互依赖关系
//!
use crate::{Args, fairing, openapi};
use rocket::data::{ByteUnit, Limits};
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
        log_level: rocket::config::LogLevel::Critical,
        ..Config::debug_default()
    });

    // 前置基础安全校验（不提取body数据）
    builder = builder.attach(fairing::security::PreSecurity::new());
    // 鉴权（前置安全校验后，提取请求数据前执行）
    builder = builder.attach(fairing::auth::Authentication::new());
    // 提取请求上下文（鉴权通过后，全局过滤器执行开始前执行）
    builder = builder.attach(fairing::request::RequestData::new());
    // 全局前置过滤器（收到请求后，到达具体API接口前执行）
    // 这些过滤器可自由配置，串联执行
    builder = builder.attach(fairing::global_filter::GlobalPreFilter::new());
    // API前置过滤器，可自由配置，串联执行
    builder = builder.attach(fairing::filter::PreFilter::new());
    // API后置过滤器，可自由配置，串联执行
    builder = builder.attach(fairing::filter::PostFilter::new());
    // 全局后置过滤器（API接口执行完成后，响应客户端前执行）
    // 这些过滤器可自由配置，串联执行
    builder = builder.attach(fairing::global_filter::GlobalPostFilter::new());
    // 设置响应（响应客户端前执行）
    builder = builder.attach(fairing::response::ResponseData::new());
    // 日志记录（响应客户端前执行）
    builder = builder.attach(fairing::logger::Logger::new());
    // 清理
    builder = builder.attach(fairing::cleanup::Cleaner::new());

    builder = builder.mount("/openapi/v1", routes![openapi::call]);

    builder.launch().await?;

    Ok(())
}
