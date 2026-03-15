use aiway_protocol::rmcp::model::{JsonRpcVersion2_0, RequestId};
use http::StatusCode;
use reqwest::Body;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_util::bytes::Bytes;

///JSON-RPC 响应
#[derive(Debug, Serialize, Deserialize)]
pub struct McpRes<T: Sized> {
    jsonrpc: JsonRpcVersion2_0,
    /// 当请求类型为通知时，无此字段
    id: Option<RequestId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<McpError>,
    /// 仅适用于通知
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<T>,
}

#[derive(Debug, Serialize, Deserialize)]
struct McpError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<String>,
}

/// 响应成功
const SUCCESS_CODE: i32 = 0;
/// 系统错误
const ERROR_CODE: i32 = 1;

impl<T> McpRes<T>
where
    T: Serialize + Sized,
{
    pub fn success(data: T, id: RequestId) -> Self {
        McpRes {
            jsonrpc: JsonRpcVersion2_0,
            id: Some(id),
            result: Some(data),
            error: None,
            params: None,
        }
    }

    pub fn notif(data: T) -> Self {
        McpRes {
            jsonrpc: JsonRpcVersion2_0,
            id: None,
            result: None,
            error: None,
            params: Some(data),
        }
    }

    pub fn error(msg: &str, id: RequestId) -> Self {
        McpRes {
            jsonrpc: JsonRpcVersion2_0,
            id: Some(id),
            result: None,
            error: Some(McpError {
                code: ERROR_CODE,
                message: msg.to_string(),
                data: None,
            }),
            params: None,
        }
    }

    #[allow(unused)]
    pub fn is_success(&self) -> bool {
        self.error.is_some()
    }

    pub fn to_json_string(&self) -> String {
        serde_json::json!(&self).to_string()
    }

    pub fn to_json_value(&self) -> Value {
        serde_json::json!(&self)
    }

    /*pub fn into_sse_stream(self) -> Pin<Box<dyn Stream<Item = Value> + Send>> {
        let stream = tokio_stream::once(Ok::<Bytes, http::Error>(
            sse_stream::Sse {
                data: self.to_json_string().into(),
                ..sse_stream::Sse::default()
            }
            .into(),
        ));
        Box::pin(stream)
    }*/
}

impl<T: Serialize> From<McpRes<T>> for reqwest::Response {
    fn from(value: McpRes<T>) -> Self {
        /*let http_resp = http::Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/event-stream")
            .body(Body::wrap_stream(tokio_stream::once(Ok::<
                Bytes,
                http::Error,
            >(
                sse_stream::Sse {
                    data: value.to_json_string().into(),
                    ..sse_stream::Sse::default()
                }
                .into(),
            ))))
            .unwrap();*/
        let http_resp = http::Response::builder()
    .status(StatusCode::OK)
    .header("Content-Type", "application/json")
    .body(value.to_json_string())
    .unwrap();
        reqwest::Response::from(http_resp)
    }
}

pub struct EmptyResponse;
impl From<EmptyResponse> for reqwest::Response {
    fn from(_value: EmptyResponse) -> Self {
        /*let http_resp = http::Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/event-stream")
            .body::<Bytes>(
                sse_stream::Sse {
                    data: Some("".into()),
                    ..sse_stream::Sse::default()
                }
                .into(),
            )
            .unwrap();*/
        let http_resp = http::Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body("")
            .unwrap();
        reqwest::Response::from(http_resp)
    }
}
