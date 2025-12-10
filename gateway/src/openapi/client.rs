use dashmap::DashMap;
use reqwest::header::{HeaderMap, HeaderName};
use reqwest::{Client, ClientBuilder, Url};
use std::str::FromStr;
use std::sync::LazyLock;

/// 对LoadBalanceClient的封装
pub struct HttpClient {
    client: Client,
}

pub static HTTP_CLIENT: LazyLock<HttpClient> = LazyLock::new(HttpClient::new);

impl HttpClient {
    pub fn new() -> Self {
        let client = ClientBuilder::default().build().unwrap();
        Self { client }
    }

    pub async fn request(
        &self,
        method: &str,
        url: Url,
        headers: DashMap<String, String>,
        body: impl Into<reqwest::Body>,
    ) -> anyhow::Result<reqwest::Result<reqwest::Response>> {
        Ok(self
            .client
            .request(reqwest::Method::from_str(method)?, url)
            .body(body)
            .headers(headers.into_header_map())
            .send()
            .await)
    }
}

pub trait IntoHeaderMap {
    /// 转换为HeaderMap
    fn into_header_map(self) -> HeaderMap;
}

impl IntoHeaderMap for DashMap<String, String> {
    fn into_header_map(self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        for item in self.iter() {
            if let (Ok(name), Ok(value)) = (
                HeaderName::from_str(item.key().as_str()),
                item.value().parse(),
            ) {
                headers.insert(name, value);
            }
        }
        headers
    }
}
