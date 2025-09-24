use std::sync::LazyLock;

#[allow(unused_must_use)]
pub static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(3))
        .read_timeout(std::time::Duration::from_secs(300))
        .build()
        .unwrap()
});
