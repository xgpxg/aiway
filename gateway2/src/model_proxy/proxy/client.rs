use crate::model_proxy::proxy::ModelError;
use reqwest::{Method, RequestBuilder, Response};
use std::collections::HashMap;
use std::pin::Pin;
use tokio_stream::Stream;

const MIME_TYPE_APPLICATION_JSON: &str = "application/json";


/// 模型客户端，参考openai_dive实现。
///
/// openai_dive存在的问题：
/// - 依赖base_url，且端点地址写死了
/// - 对于部分模型提供商的API是非标准OpenAI格式，接口路径可能对不上
/// - 不方便扩展
///
/// 所以需要能够直接请求提供商的API地址，而不是OpenAI标准的地址。
pub struct Client {
    /// HTTP客户端
    http_client: reqwest::Client,
    /// 模型提供商的API密钥
    api_key: Option<String>,
}

impl Client {
    pub fn new(api_key: Option<String>) -> Self {
        Client {
            http_client: reqwest::Client::new(),
            api_key,
        }
    }

    fn build_request(
        &self,
        method: Method,
        url: &str,
        content_type: Option<&str>,
    ) -> RequestBuilder {
        let mut request = self.http_client.request(method, url);
        if let Some(api_key) = &self.api_key {
            request = request.bearer_auth(api_key);
        }

        if let Some(content_type) = content_type {
            request = request.header(reqwest::header::CONTENT_TYPE, content_type);
        }

        request
    }

    pub(crate) async fn post<I, Q>(
        &self,
        url: &str,
        body: I,
        query: Q,
    ) -> Result<Response, ModelError>
    where
        I: Into<reqwest::Body>,
        Q: Into<Option<HashMap<String, String>>>,
    {
        let response = self
            .build_request(Method::POST, url, Some(MIME_TYPE_APPLICATION_JSON))
            .query(&query.into())
            .body(body.into())
            .send()
            .await;

        response.map_err(|error| ModelError::RequestProviderError(error.to_string()))
    }
}
