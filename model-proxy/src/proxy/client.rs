use aha_reqwest_eventsource::{Event, EventSource, RequestBuilderExt};
use bytes::Bytes;
use openai_dive::v1::error::{APIError, InvalidRequestError};
use reqwest::{Method, RequestBuilder, Response, StatusCode};
use rocket::futures::Stream;
use rocket::serde::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;
use std::pin::Pin;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::UnboundedReceiverStream;

const MIME_TYPE_APPLICATION_JSON: &str = "application/json";

type ModelStream<O> = Pin<Box<dyn Stream<Item = Result<O, APIError>> + Send>>;

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

    pub(crate) fn format_response<R: DeserializeOwned>(response: String) -> Result<R, APIError> {
        let value = Self::validate_response(response)?;

        let value: R = serde_json::from_value(value)
            .map_err(|error| APIError::ParseError(error.to_string()))?;

        Ok(value)
    }

    pub(crate) fn validate_response(response: String) -> Result<Value, APIError> {
        let value: Value = serde_json::from_str(&response)
            .map_err(|error| APIError::ParseError(error.to_string()))?;

        if let Some(object) = value.as_object()
            && object.len() == 1
            && object.contains_key("error")
        {
            return Err(APIError::InvalidRequestError(value["error"].to_string()));
        }

        Ok(value)
    }
    pub(crate) async fn check_status_code(
        result: reqwest::Result<Response>,
    ) -> Result<Response, APIError> {
        match result {
            Ok(response) => {
                if response.status().is_client_error() {
                    let status = response.status();

                    let text = response
                        .text()
                        .await
                        .map_err(|error| APIError::ParseError(error.to_string()))?;

                    return match status {
                        StatusCode::BAD_REQUEST => Err(APIError::BadRequestError(text)),
                        StatusCode::UNAUTHORIZED => Err(APIError::AuthenticationError(text)),
                        StatusCode::FORBIDDEN => Err(APIError::PermissionError(text)),
                        StatusCode::NOT_FOUND => Err(APIError::NotFoundError(text)),
                        StatusCode::GONE => Err(APIError::GoneError(text)),
                        StatusCode::TOO_MANY_REQUESTS => Err(APIError::RateLimitError(text)),
                        _ => Err(APIError::UnknownError(status.as_u16(), text)),
                    };
                }

                Ok(response)
            }
            Err(error) => Err(APIError::ServerError(error.to_string())),
        }
    }
    pub async fn post<I, O, Q>(&self, url: &str, body: I, query: Q) -> Result<O, APIError>
    where
        I: Into<reqwest::Body>,
        O: DeserializeOwned,
        Q: Into<Option<HashMap<String, String>>>,
    {
        let response = self
            .build_request(Method::POST, url, Some(MIME_TYPE_APPLICATION_JSON))
            .query(&query.into())
            .body(body.into())
            .send()
            .await;

        let response = Self::check_status_code(response).await?;

        let response_text = response
            .text()
            .await
            .map_err(|error| APIError::ParseError(error.to_string()))?;

        Self::format_response(response_text)
    }

    pub async fn post_raw<I, Q>(&self, url: &str, body: I, query: Q) -> Result<Bytes, APIError>
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
        let response = match Self::check_status_code(response).await {
            Ok(response) => response,
            Err(error) => return Err(error),
        };
        let bytes = response
            .bytes()
            .await
            .map_err(|error| APIError::ParseError(error.to_string()))?;
        Ok(bytes)
    }

    pub async fn post_stream<I, O, Q>(&self, url: &str, body: I, query: Q) -> ModelStream<O>
    where
        I: Into<reqwest::Body>,
        O: DeserializeOwned + Send + 'static,
        Q: Into<Option<HashMap<String, String>>>,
    {
        let event_source = self
            .build_request(Method::POST, url, Some(MIME_TYPE_APPLICATION_JSON))
            .query(&query.into())
            .body(body.into())
            .eventsource()
            .unwrap();

        Self::process_stream::<O>(event_source).await
    }

    pub(crate) async fn process_stream<O>(mut event_source: EventSource) -> ModelStream<O>
    where
        O: DeserializeOwned + Send + 'static,
    {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn(async move {
            while let Some(event_result) = event_source.next().await {
                match event_result {
                    Ok(event) => match event {
                        Event::Open => continue,
                        Event::Message(message) => {
                            if message.data == "[DONE]" {
                                break;
                            }

                            let response = match serde_json::from_str::<O>(&message.data) {
                                Ok(result) => Ok(result),
                                Err(error) => {
                                    match serde_json::from_str::<InvalidRequestError>(&message.data)
                                    {
                                        Ok(invalid_request_error) => Err(APIError::StreamError(
                                            invalid_request_error.to_string(),
                                        )),
                                        Err(_) => Err(APIError::StreamError(format!(
                                            "{} {}",
                                            error, message.data
                                        ))),
                                    }
                                }
                            };

                            if let Err(_error) = tx.send(response) {
                                break;
                            }
                        }
                    },
                    Err(error) => {
                        if let Err(_error) = tx.send(Err(APIError::StreamError(error.to_string())))
                        {
                            break;
                        }
                    }
                }
            }

            event_source.close();
        });

        Box::pin(UnboundedReceiverStream::new(rx))
    }
}
