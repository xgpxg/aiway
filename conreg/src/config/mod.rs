pub mod server;

use crate::Args;
use logging::log;
use std::process::exit;

use crate::config::server::{ConfigApp, ConfigManager};

pub async fn new_config_app(args: &Args) -> ConfigApp {

    let manager = ConfigManager::new(args).await;
    if let Err(e) = manager {
        log::error!("Failed to create config app: {}", e);
        exit(1);
    }
    ConfigApp {
        manager: manager.unwrap(),
    }
}
