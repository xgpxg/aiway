use crate::openapi::error::GatewayError;
use rocket::Request;
use rocket::response::Responder;
use rocket::serde::json::serde_json;
use tokio::io::AsyncRead;

pub enum GatewayResponse {
    /// JSON响应
    Json(serde_json::Value),
    /// 流式响应，以纯文本返回
    Stream(Box<dyn AsyncRead + Unpin + Send>),
    /// SSE响应，以SSE格式返回
    SSE(Box<dyn AsyncRead + Unpin + Send>),
    /// 错误响应
    Error(GatewayError),
}

impl<'r> Responder<'r, 'r> for GatewayResponse {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'r> {
        match self {
            GatewayResponse::Json(data) => {
                let json = serde_json::to_string(&data).unwrap();
                rocket::response::Response::build()
                    .header(rocket::http::ContentType::JSON)
                    .sized_body(json.len(), std::io::Cursor::new(json))
                    .ok()
            }
            GatewayResponse::Stream(reader) => rocket::response::Response::build()
                .header(rocket::http::ContentType::Plain)
                .streamed_body(reader)
                .ok(),
            GatewayResponse::SSE(reader) => rocket::response::Response::build()
                .header(rocket::http::ContentType::EventStream)
                .streamed_body(reader)
                .ok(),
            GatewayResponse::Error(e) => e.respond_to(request),
        }
    }
}
