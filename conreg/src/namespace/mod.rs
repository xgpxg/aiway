pub mod server;

use crate::Args;
use crate::namespace::server::{NamespaceApp, NamespaceManager};
use logging::log;
use std::process::exit;


pub async fn new_namespace_app(args: &Args) -> NamespaceApp {
    let manager = NamespaceManager::new(args).await;
    if let Err(e) = manager {
        log::error!("Failed to create namespace app: {}", e);
        exit(1);
    }
    NamespaceApp {
        manager: manager.unwrap(),
    }
}
