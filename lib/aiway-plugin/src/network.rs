use reqwest::{Client, ClientBuilder};
use std::sync::LazyLock;

pub(crate) struct Network {
    pub(crate) client: Client,
}
impl Network {
    pub fn new() -> Self {
        let client = ClientBuilder::default()
            .connect_timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap();
        Self { client }
    }
}

pub(crate) static NETWORK: LazyLock<Network> = LazyLock::new(Network::new);
