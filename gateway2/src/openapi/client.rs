//! # HTTP 客户端
//!
use reqwest::{Client, Method, Url};
use http::HeaderMap;
use bytes::Bytes;

pub static HTTP_CLIENT: once_cell::sync::Lazy<Client> = 
    once_cell::sync::Lazy::new(|| Client::builder().build().unwrap());

pub async fn request(
    method: Method,
    url: Url,
    headers: HeaderMap,
    body: Bytes,
) -> Result<reqwest::Response, reqwest::Error> {
    let mut builder = HTTP_CLIENT.request(method, url);
    
    for (key, value) in headers.iter() {
        builder = builder.header(key, value);
    }
    
    builder.body(body.to_vec()).send().await
}
