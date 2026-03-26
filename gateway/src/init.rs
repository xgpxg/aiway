//! # 初始化模块
//!
use crate::Args;
use crate::components::{
    ApiKeySyncer, ConfigFactory, Firewalld, GlobalPluginFactory, IpRegion, Router, Servicer,
};
use crate::report;
use crate::report::STATE;
use alert::Alert;
use clap::Parser;
use logging::LogAppender;
use std::path::PathBuf;
use std::sync::LazyLock;

pub async fn init(args: &Args) {
    // 初始化日志
    logging::init_log_with(
        LogAppender::CONSOLE | LogAppender::QUICKWIT,
        logging::Config {
            quickwit_endpoint: Some(args.log_server.clone()),
            ..Default::default()
        },
    );

    // 初始化缓存
    cache::init_local_cache(cache_dir(args)).unwrap();

    // 初始全局路由过滤器配置
    GlobalPluginFactory::init().await;

    // 初始化插件管理器
    plugin_manager::init(&args.console).await;

    // 初始化路由
    Router::init().await;

    // 初始化服务
    Servicer::init().await;

    // 初始化防火墙
    Firewalld::init().await;

    // 初始化 IpRegion
    IpRegion::init().await;

    // 初始化配置
    ConfigFactory::init().await;

    // 初始化监控
    report::init(args);

    // 初始化告警
    alert::init(args.console.clone());

    // 设置 panic hook
    set_panic_hook();

    // 同步 APIKey
    ApiKeySyncer::init().await;

    #[cfg(feature = "model-proxy")]
    {
        crate::model_proxy::init(args).await;
    }

    #[cfg(feature = "mcp-proxy")]
    {
        crate::mcp_proxy::init(args).await;
    }
}

fn cache_dir(args: &Args) -> PathBuf {
    common::dir::AppDir::cache_dir()
        .join("gateway")
        .join(args.port.to_string())
}

fn set_panic_hook() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        log::error!("{}", info);

        STATE.inc_status_request_count(500, 1);
        STATE.inc_http_connect_count(-1);

        alert_error("网关节点出现Panic，请关注！", &info.to_string());

        hook(info);
    }));
}

static ARGS: LazyLock<Args> = LazyLock::new(|| Args::parse());
pub fn alert_error(title: &str, content: &str) {
    let args = &*ARGS;
    let full_content = format!("{}\n节点地址：{}:{}", content, args.address, args.port);
    Alert::error(title, &full_content);
}
pub fn alert_warn(title: &str, content: &str) {
    let args = &*ARGS;
    let full_content = format!("{}\n节点地址：{}:{}", content, args.address, args.port);
    Alert::warn(title, &full_content);
}
