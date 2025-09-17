use crate::Args;
use conreg_client::conf::{ClientConfigBuilder, ConRegConfigBuilder, DiscoveryConfigBuilder};
use conreg_client::init_with;

pub async fn init(args: &Args) {
    logging::init_log();
    init_client(args).await;
}

async fn init_client(args: &Args) {
    let config = ConRegConfigBuilder::default()
        .client(
            ClientConfigBuilder::default()
                .port(args.port)
                .build()
                .unwrap(),
        )
        .discovery(
            DiscoveryConfigBuilder::default()
                .server_addr("127.0.0.1:8000")
                .build()
                .unwrap(),
        )
        .build()
        .unwrap();

    init_with(config).await;
}
