//! # 网关接入层
//! 用于提供给外部接入，可将域名解析到本服务
//!
//! L4 透传 + TLS 终止

use clap::Parser;
use logging::{LogAppender, init_log_with};
use proxy::{AccessPoint, NodePool};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;

mod cert;
mod proxy;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Listen address
    #[arg(short, long, default_value = "0.0.0.0")]
    pub address: String,

    /// HTTP listen port
    #[arg(short, long, default_value_t = 7080)]
    pub port: u16,

    /// HTTPS listen port, 0 means disabled
    #[arg(long, default_value_t = 0)]
    pub https_port: u16,

    /// Console address
    #[arg(short, long, default_value = "127.0.0.1:7000")]
    pub console: String,

    /// Log server address
    #[arg(short, long, default_value = "127.0.0.1:7280")]
    pub log_server: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // 初始化日志
    init_log_with(
        LogAppender::CONSOLE,
        logging::Config {
            quickwit_endpoint: Some(args.log_server.clone()),
            ..Default::default()
        },
    );

    log::info!(
        "access v{} starting, listening on {}:{}",
        VERSION,
        args.address,
        args.port
    );

    // 初始化证书管理器（如果配置了 HTTPS 端口）
    if args.https_port > 0 {
        cert::CertManager::init(&args.console).await;
    }

    // 从控制台获取初始节点列表
    let initial_nodes = fetch_nodes(&args.console).await?;
    log::info!("Loaded {} gateway nodes", initial_nodes.len());

    let node_pool = Arc::new(NodePool::new(initial_nodes));

    // 启动节点列表刷新任务
    {
        let console_addr = args.console.clone();
        let pool = node_pool.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            loop {
                interval.tick().await;
                match fetch_nodes(&console_addr).await {
                    Ok(nodes) => {
                        let old_len = pool.len();
                        if nodes.len() != old_len {
                            log::info!("Gateway nodes changed: {} -> {}", old_len, nodes.len());
                        }
                        pool.update(nodes);
                    }
                    Err(e) => {
                        log::error!("Failed to fetch gateway nodes: {}", e);
                    }
                }
            }
        });
    }

    let access_point = Arc::new(AccessPoint::new(node_pool));

    // 启动 HTTP 监听（L4 透传）
    {
        let addr = format!("{}:{}", args.address, args.port);
        let listener = TcpListener::bind(&addr).await?;
        let ap = access_point.clone();

        log::info!("HTTP (L4) listening on {}", addr);

        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, peer_addr)) => {
                        let ap = ap.clone();
                        tokio::spawn(async move {
                            ap.handle_connection(stream, peer_addr).await;
                        });
                    }
                    Err(e) => {
                        log::error!("HTTP accept error: {}", e);
                    }
                }
            }
        });
    }

    // 启动 HTTPS 监听（TLS 终止 + L4 透传）
    if args.https_port > 0 {
        let addr = format!("{}:{}", args.address, args.https_port);
        let listener = TcpListener::bind(&addr).await?;
        let ap = access_point.clone();

        log::info!(
            "HTTPS (TLS + L4) listening on {} with SNI-based certificate selection",
            addr
        );

        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, peer_addr)) => {
                        let ap = ap.clone();
                        tokio::spawn(async move {
                            // TLS 终止
                            let tls_stream = match cert::tls_accept(stream, peer_addr).await {
                                Ok(s) => s,
                                Err(e) => {
                                    log::debug!("[{}] TLS handshake failed: {}", peer_addr, e);
                                    return;
                                }
                            };
                            ap.handle_connection(tls_stream, peer_addr).await;
                        });
                    }
                    Err(e) => {
                        log::error!("HTTPS accept error: {}", e);
                    }
                }
            }
        });
    }

    log::info!("access started, ready to accept connections");

    // 等待退出信号
    tokio::signal::ctrl_c().await?;
    log::info!("access shutting down");

    Ok(())
}

/// 从控制台获取在线的网关节点列表
async fn fetch_nodes(
    console: &str,
) -> anyhow::Result<Vec<aiway_protocol::gateway::GatewayNodeInfo>> {
    let url = format!("http://{}/api/v1/gateway/online-nodes", console);
    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        anyhow::bail!("Console returned error: {}", response.status());
    }

    let res = response
        .json::<busi::res::Res<Vec<aiway_protocol::gateway::GatewayNodeInfo>>>()
        .await?;

    if res.is_success() {
        res.data.ok_or_else(|| anyhow::anyhow!("No data returned"))
    } else {
        anyhow::bail!("Console returned error: {}", res.msg)
    }
}
