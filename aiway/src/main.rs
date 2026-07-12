mod embed;

use clap::Parser;
use logging::{init_log, log};
use rust_embed::Embed;
use std::thread::sleep;
use std::time::Duration;

#[derive(Embed)]
#[folder = "bin/"]
pub(crate) struct Asset;

struct AiwayApp {
    // 日志服务
    logg: embed::EmbedApp,
    // 网关应用
    gateway: embed::EmbedApp,
    // 控制台应用
    console: embed::EmbedApp,
    // 接入层
    access: embed::EmbedApp,
}

impl AiwayApp {
    fn new(args: &Args) -> Self {
        let console = Asset::get("console").unwrap();
        let gateway = Asset::get("gateway").unwrap();
        let logg = Asset::get("logg").unwrap();
        let access = Asset::get("access").unwrap();

        let logg = embed::EmbedApp::new("logg", &logg.data, &[]).unwrap();
        log::info!("log server started");

        let console = embed::EmbedApp::new(
            "console",
            &console.data,
            &[
                "--address",
                &args.console_address,
                "--port",
                &args.console_port.to_string(),
                "--log-server",
                &args.log_server,
            ],
        )
        .unwrap();
        log::info!("console started");

        // 等待console启动完成
        // 这里实现 不优雅，先这样，后续处理
        sleep(Duration::from_secs(2));

        let gateway = embed::EmbedApp::new(
            "gateway",
            &gateway.data,
            &[
                "--address",
                &args.gateway_address,
                "--port",
                &args.gateway_port.to_string(),
                "--console",
                &format!("{}:{}", args.console_address, args.console_port),
                "--log-server",
                &args.log_server.to_string(),
            ],
        )
        .unwrap();
        log::info!("gateway started");

        let access = embed::EmbedApp::new(
            "access",
            &access.data,
            &[
                "--address",
                &args.access_address,
                "--port",
                &args.access_http_port.to_string(),
                "--https-port",
                &args.access_https_port.to_string(),
                "--log-server",
                &args.log_server,
            ],
        )
        .unwrap();

        AiwayApp {
            logg,
            console,
            gateway,
            access,
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Console listen address
    #[arg(long, default_value = "127.0.0.1")]
    console_address: String,

    /// Console listen port
    #[arg(long, default_value_t = 7000)]
    console_port: u16,

    /// Gateway listen address
    #[arg(long, default_value = "127.0.0.1")]
    gateway_address: String,

    /// Gateway listen port
    #[arg(long, default_value_t = 7001)]
    gateway_port: u16,

    /// Log server address
    #[arg(long, default_value = "127.0.0.1:7280")]
    log_server: String,

    /// Access listen address
    #[arg(long, default_value = "0.0.0.0")]
    access_address: String,

    /// Access http listen port
    #[arg(long, default_value_t = 7080)]
    access_http_port: u16,

    /// Access https listen port
    #[arg(long, default_value_t = 7443)]
    access_https_port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    init_log();

    /* tokio::spawn(async {
        start_share_cache_server(AppDir::cache_dir()).await.unwrap();
    });*/

    let _app = AiwayApp::new(&args);

    tokio::signal::ctrl_c().await?;

    Ok(())
}
