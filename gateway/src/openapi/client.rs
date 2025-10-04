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

    /// get请求
    ///
    /// - url 请求地址。注意是lb地址
    /// - headers 请求头
    pub async fn get(
        &self,
        url: Url,
        headers: DashMap<String, String>,
    ) -> anyhow::Result<reqwest::Result<reqwest::Response>> {
        Ok(self
            .client
            .get(url)
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
    /// DashMap转换为HeaderMap
    fn into_header_map(self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        for item in self.iter() {
            headers.insert(
                HeaderName::from_str(item.key().as_str()).unwrap(),
                item.value().parse().unwrap(),
            );
        }
        headers
    }
}

