use logging::{init_log, log};
use rust_embed::Embed;
use std::env::temp_dir;
use std::path::PathBuf;
use std::process::{Child, Command};

#[derive(Embed)]
#[folder = "bin/"]
struct Asset;

struct NatsServer {
    child: Option<Child>,
    binary_path: PathBuf,
}

impl NatsServer {
    fn new(binary_data: &[u8], args: &[&str]) -> Result<Self, Box<dyn std::error::Error>> {
        let path = temp_dir().join("nats-server");
        std::fs::write(&path, binary_data)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&path, perms)?;
        }

        let child = Command::new(&path).args(args).spawn()?;

        log::info!("Process started with PID: {}", child.id());

        Ok(NatsServer {
            child: Some(child),
            binary_path: path,
        })
    }

    fn kill(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mut child) = self.child.take() {
            child.kill()?;
            let _ = child.wait();
        }
        Ok(())
    }
}

impl Drop for NatsServer {
    fn drop(&mut self) {
        let _ = self.kill();
        let _ = std::fs::remove_file(&self.binary_path);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_log();
    let nats_server = Asset::get("nats-server").unwrap();

    let _process = NatsServer::new(&nats_server.data, &["-p", "4222", "-js"])?;

    log::info!("NATS server is running. Press Enter to stop...");

    tokio::signal::ctrl_c().await?;

    Ok(())
}
