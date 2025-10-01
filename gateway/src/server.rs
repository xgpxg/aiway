//! # 网关服务端
//!
//! ## 基本准则
//! - 精简实现，不要耦合任何复杂的业务逻辑，具体逻辑应由插件实现
//! - 插件化，通过可序列化的数据进行通信，同一类型的插件接口参数应保持一致
//! - 涉及到网络连接的，需池化、复用，避免频繁创建、销毁连接
//! - 网关应不依赖任何中间件，可水平扩展，每个节点需独立运行，无相互依赖关系
//! - 网关服务允许访问外部系统，但外部系统不一定能访问到网关，需要注意网络连通性。
//!
//! ## 节点注册
//! 网关节点启动时将自身注册到控制台，控制台应将节点信息持久化保存。
//! 如果注册失败，则应该退出进程。
//!
//! 网关节点应定时发送心跳，向控制台同步最新状态，即使控制台无法连接，网关也应正常运行。
//! 控制台超时未收到心跳时，也不应删除该节点信息。
//!
//! 控制台地址通过启动参数传入。
//!
//! 目前，控制台设计为单机模式，因为控制台仅作为管理工具，不会影响正在运行的网关节点，
//! 也就是说，在控制台宕机的情况下，网关节点仍可正常运行（重启后除外）。
//! 另一方面，控制台单机理论上能够支持1w+ qps，完全可满足1k台以内网关节点的数据同步和心跳请求，
//! 性能方面单机即可满足，并且控制台依赖于关系型数据库以及Redis的之持久化，能满足数据一致性的要求,
//! 所以没必要集群。
//!
//! ## 配置同步
//! 由于网关本身不存储配置，不需要保持强一致性，保证最终一致性即可，
//! 即使配置变更过程中，有秒级延时也是可以接受的。
//!
//! ## 日志
//! 日志作为网关重要的数据分析来源，应单独存储，并支持查询。
//!
//! 计划使用[quickwit](https://github.com/quickwit-oss/quickwit)作为日志存储和搜索服务。
//!
//!
//!
use crate::openapi::eep;
use crate::{Args, fairing, openapi};
use rocket::data::{ByteUnit, Limits};
use rocket::fairing::AdHoc;
use rocket::{Config, catchers, routes};
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

    ////////////////////////////////// 请求阶段 //////////////////////////////////
    // 前置基础安全校验，内置实现，暂不支持扩展，仅校验基本参数，不提取body数据验证。
    builder = builder.attach(fairing::security::PreSecurity::new());
    // 鉴权，即验证API Key
    builder = builder.attach(fairing::auth::Authentication::new());
    // 提取请求上下文，放在鉴权之后，防止无效的上下文提取
    builder = builder.attach(fairing::request::RequestData::new());
    // 全局前置过滤器，可自由配置，串联执行，对整个网关生效
    builder = builder.attach(fairing::global_filter::GlobalPreFilter::new());
    // 路由匹配
    builder = builder.attach(fairing::routing::Routing::new());
    // 路由前置过滤器，可自由配置，串联执行，对单个路由生效，由于插件本身要求设计为无状态，所以，理论上各个路由的相同插件互不影响
    // 注意：是在路由匹配之后执行，因为要先匹配到路由，才能获取路由对应的插件，这点可能和命名有点歧义。
    builder = builder.attach(fairing::filter::PreFilter::new());
    // 负载均衡，通过路由配置对应的服务，进行负载，然后路由到具体的服务执行
    builder = builder.attach(fairing::lb::LoadBalance::new());

    ////////////////////////////////// 响应阶段 //////////////////////////////////
    // 路由后置过滤器，可自由配置，串联执行
    builder = builder.attach(fairing::filter::PostFilter::new());
    // 全局后置过滤器（API接口执行完成后，响应客户端前执行），可自由配置，串联执行
    builder = builder.attach(fairing::global_filter::GlobalPostFilter::new());
    // 设置响应，必须执行
    builder = builder.attach(fairing::response::ResponseData::new());
    // 日志记录，必须执行
    builder = builder.attach(fairing::logger::Logger::new());
    // 清理，必须执行
    builder = builder.attach(fairing::cleanup::Cleaner::new());

    builder = builder.mount("/openapi/v1", routes![openapi::call]);
    builder = builder.mount("/eep", eep::routes());

    builder = builder.attach(AdHoc::on_liftoff("Print Banner", |_| {
        Box::pin(async {
            print_banner();
        })
    }));

    builder = builder.register(
        "/",
        catchers![
            fairing::catchers::catch_401,
            fairing::catchers::catch_403,
            fairing::catchers::catch_404
        ],
    );

    builder.launch().await?;

    Ok(())
}

fn print_banner() {
    use clap::Parser;
    let args = Args::parse();
    log::info!(
        "aiway started success, current version: {}, listening on: {}:{}",
        crate::VERSION,
        args.address,
        args.port
    );
}
