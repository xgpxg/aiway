mod error;
mod response;
mod router;
mod sse;

use crate::context::{RCM, RequestContextWrapper};
use crate::openapi::response::GatewayResponse;
use crate::openapi::sse::SseEvent;
use protocol::gateway::RequestContext;
use rocket::async_stream::stream;
use rocket::http::uri::fmt::Path;
use rocket::request::FromSegments;
use rocket::{get, post};
use std::io;
use std::io::Bytes;
use std::path::PathBuf;
use std::pin::Pin;
use tokio_util::io::StreamReader;

/// OpenAPI统一入口
///
/// 要求：
/// - 所有请求都必须有响应，无论成功或失败
/// - 接口内部不处理任何业务逻辑，需转发到具体服务上处理
/// - 同时支持流式和非流式响应
/// - 流式响应支持恢复（插件实现）
#[post("/<path..>")]
pub async fn call(wrapper: RequestContextWrapper, path: PathBuf) -> GatewayResponse {
    let context = wrapper.0;

    // 1. 通过path获取端点配置

    // 2. 获取负载实例、协议类型

    // 3. 转发请求

    // 4. 获取响应

    // 5. 封装响应

    // 6. 返回

    let stream = stream! {
        for i in 0..10 {
            let data = SseEvent::Data(format!("hello world {}", i)).to_string().into_bytes();
            yield Ok::<_, io::Error>(io::Cursor::new(data));
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
    };
    let stream_reader = Box::new(StreamReader::new(Box::pin(stream)));

    // Json响应
    // GatewayResponse::Json(context.get_query().into())

    // SSE响应
    GatewayResponse::SSE(stream_reader)
}
