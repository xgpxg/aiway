//! # 网关响应定义
//! 执行顺序：respond_to -> response fairing
use crate::openapi::error::GatewayError;
use crate::report::STATE;
use protocol::gateway::ResponseContext;
use reqwest::header;
use rocket::Request;
use rocket::futures::stream;
use rocket::response::Responder;
use tokio_util::bytes::Bytes;

pub enum GatewayResponse {
    Success,
    /// 错误响应
    Error(GatewayError),
}

impl<'r> Responder<'r, 'r> for GatewayResponse {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'r> {
        match self {
            GatewayResponse::Success => rocket::response::Response::build().ok(),
            GatewayResponse::Error(e) => e.respond_to(request),
        }
    }
}

pub trait ResponseExt {
    fn is_sse(&self) -> bool;

    async fn into_context(self, context: &ResponseContext);
}
impl ResponseExt for reqwest::Response {
    #[inline]
    fn is_sse(&self) -> bool {
        if let Some(content_type) = self.headers().get(header::CONTENT_TYPE) {
            return content_type.as_bytes().starts_with(b"text/event-stream");
        }
        false
    }

    async fn into_context(self, context: &ResponseContext) {
        use rocket::futures::StreamExt;
        // 设置状态码
        context.set_status(self.status().as_u16());

        // 设置响应头
        context.set_headers(self.headers().iter().map(|(k, v)| {
            (
                k.as_str().to_owned(),
                v.to_str()
                    .map(|s| s.to_owned())
                    .unwrap_or_else(|_| String::from_utf8_lossy(v.as_bytes()).to_string()),
            )
        }));

        // 处理SSE流
        if self.is_sse() {
            // 处理SSE流结束，将这个流合并在响应流的最后
            let end_handler = stream::once(async {
                // SSE连接数减1，这里不用处理HTTP连接数，会在cleanup中处理
                STATE.inc_sse_connect_count(-1);
                Ok(Bytes::new())
            });

            let stream = self
                .bytes_stream()
                .chain(end_handler)
                .map(|item| match item {
                    Ok(item) => Ok(item.to_vec()),
                    Err(e) => Err(e.into()),
                });
            context.set_stream_body(Box::pin(stream));

            return;
        }
        context.set_body(self.bytes().await.unwrap());
    }
}
