mod embed;

use cache::start_share_cache_server;
use logging::init_log;
use rust_embed::Embed;
use std::thread::sleep;
use std::time::Duration;

#[derive(Embed)]
#[folder = "bin/"]
pub(crate) struct Asset;

struct AiwayApp {
    // 网关应用
    gateway: embed::EmbedApp,
    // 控制台应用
    console: embed::EmbedApp,
}

impl AiwayApp {
    fn new() -> Self {
        let console = Asset::get("console").unwrap();
        let gateway = Asset::get("gateway").unwrap();
        let console = embed::EmbedApp::new("console", &console.data, &[]).unwrap();
        // 等待console启动完成
        // 这里实现 不优雅，先这样，后续处理
        sleep(Duration::from_secs(1));
        let gateway = embed::EmbedApp::new("gateway", &gateway.data, &[]).unwrap();
        AiwayApp { console, gateway }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_log();
    tokio::spawn(async {
        start_share_cache_server("cache").await.unwrap();
    });

    let _app = AiwayApp::new();

    tokio::signal::ctrl_c().await?;

    Ok(())
}
