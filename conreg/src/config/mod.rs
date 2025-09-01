pub mod server;

use crate::Args;

use crate::config::server::ConfigApp;

pub async fn new_config_app(args: &Args) -> ConfigApp {
    server::new_config_app(args).await
}
