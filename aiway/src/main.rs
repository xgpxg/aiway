mod embed;

use cache::start_share_cache_server;
use clap::Parser;
use common::dir::AppDir;
use logging::{init_log, log};
use rust_embed::Embed;
use std::thread::sleep;
use std::time::Duration;

#[derive(Embed)]
#[folder = "bin/"]
pub(crate) struct Asset;

struct AiwayApp {
    // 日志服务
    #[allow(unused)]
    logg: embed::EmbedApp,
    // 网关应用
    #[allow(unused)]
    gateway: embed::EmbedApp,
    // 控制台应用
    #[allow(unused)]
    console: embed::EmbedApp,
    // 模型代理
    #[allow(unused)]
    model_proxy: embed::EmbedApp,
}

impl AiwayApp {
    fn new(args: &Args) -> Self {
        let console = Asset::get("console").unwrap();
        let gateway = Asset::get("gateway").unwrap();
        let logg = Asset::get("logg").unwrap();
        let model_proxy = Asset::get("model-proxy").unwrap();

        let logg = embed::EmbedApp::new("logg", &logg.data, &[]).unwrap();
        log::info!("log server started");

        let console = embed::EmbedApp::new(
            "console",
            &console.data,
            &[
                "--port",
                &args.port.to_string(),
                "--log-server",
                "127.0.0.1:7281",
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
            &["--log-server", "127.0.0.1:7281"],
        )
        .unwrap();
        log::info!("gateway started");

        let model_proxy = embed::EmbedApp::new(
            "model-proxy",
            &model_proxy.data,
            &["--log-server", "127.0.0.1:7281"],
        )
        .unwrap();
        log::info!("model-proxy started");

        AiwayApp {
            logg,
            console,
            gateway,
            model_proxy,
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// IP address, like 127.0.0.1
    #[arg(short, long, default_value = "127.0.0.1")]
    address: String,

    /// Port
    #[arg(short, long, default_value_t = 7000)]
    port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    init_log();
    tokio::spawn(async {
        start_share_cache_server(AppDir::cache_dir()).await.unwrap();
    });

    let _app = AiwayApp::new(&args);

    tokio::signal::ctrl_c().await?;

    Ok(())
}
