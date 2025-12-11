mod embed;

use cache::start_share_cache_server;
use logging::{init_log, log};
use rust_embed::Embed;
use std::ops::Deref;
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
}

impl AiwayApp {
    fn new() -> Self {
        let console = Asset::get(BinaryName("console").as_ref()).unwrap();
        let gateway = Asset::get(BinaryName("gateway").as_ref()).unwrap();
        let logg = Asset::get(BinaryName("logg").as_ref()).unwrap();
        let logg = embed::EmbedApp::new(BinaryName("logg").as_ref(), &logg.data, &[]).unwrap();
        log::info!("log server started");
        let console = embed::EmbedApp::new(
            BinaryName("console").as_ref(),
            &console.data,
            &["--log-server", "127.0.0.1:7281"],
        )
        .unwrap();
        log::info!("console started");
        // 等待console启动完成
        // 这里实现 不优雅，先这样，后续处理
        sleep(Duration::from_secs(2));
        let gateway = embed::EmbedApp::new(
            BinaryName("gateway").as_ref(),
            &gateway.data,
            &["--log-server", "127.0.0.1:7281"],
        )
        .unwrap();
        log::info!("gateway started");
        AiwayApp {
            logg,
            console,
            gateway,
        }
    }
}

pub struct BinaryName<'a>(&'a str);
impl std::fmt::Display for BinaryName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if cfg!(windows) {
            write!(f, "{}.exe", self.0)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

impl AsRef<str> for BinaryName {
    fn as_ref(&self) -> &str {
        self.0
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
