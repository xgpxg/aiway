use crate::openapi::client::HTTP_CLIENT;
use context::HttpContextWrapper;
use reqwest::Url;
use rocket::futures::{SinkExt, StreamExt, TryStreamExt};
use rocket::{Data, get, post};
use rocket_ws::frame::{CloseCode, CloseFrame};
use std::path::PathBuf;
use tokio::select;

#[get("/ws/<path..>", data = "<data>")]
pub async fn call_get_websocket(
    ws: rocket_ws::WebSocket,
    wrapper: HttpContextWrapper,
    path: PathBuf,
    data: Data<'_>,
) -> rocket_ws::Channel<'static> {
    call_websocket(ws, wrapper, path, data).await
}

#[post("/ws/<path..>", data = "<data>")]
pub async fn call_post_websocket(
    ws: rocket_ws::WebSocket,
    wrapper: HttpContextWrapper,
    path: PathBuf,
    data: Data<'_>,
) -> rocket_ws::Channel<'static> {
    call_websocket(ws, wrapper, path, data).await
}

async fn call_websocket(
    ws: rocket_ws::WebSocket,
    wrapper: HttpContextWrapper,
    _path: PathBuf,
    _data: Data<'_>,
) -> rocket_ws::Channel<'static> {
    let request_context = &wrapper.0.clone().request;

    // 实际路由路径
    let path = &request_context.get_path().clone();

    // 路由的实际地址，该地址已经由负载均衡处理过，可能是IP或域名
    let routing_url = request_context.get_routing_url().unwrap();
    let mut url = match Url::parse(&format!(
        "{}/{}",
        routing_url.trim_end_matches('/'),
        path.trim_start_matches("/")
    )) {
        Ok(url) => url,
        // 理论上不会执行到这里
        Err(e) => {
            log::error!("parse load balance url error: {}", e);
            panic!("parse load balance url error: {}", e);
        }
    };

    // 添加query参数，如果有的话
    {
        let mut query_pairs = url.query_pairs_mut();
        query_pairs.clear();
        for q in request_context.query.iter() {
            query_pairs.append_pair(q.key(), q.value());
        }
    }

    // 请求头
    let headers = request_context.headers.clone();

    // 请求方法
    let method = request_context.get_method().unwrap_or_default().to_string();

    // 这里clone可能有性能问题
    let body = request_context.get_body().cloned();

    ws.channel(move |mut stream| {
        Box::pin(async move {
            let response = HTTP_CLIENT
                .request_ws(&method, url, headers, body.unwrap_or_default())
                .await;

            if  response.is_err() {
                let _ = stream.close(Some(CloseFrame {
                    code: CloseCode::Error,
                    reason: "Response Error".into(),
                })).await;
                return Ok(());
            }

            // SAFETY: 上面已经验证
            let response = response.unwrap();

            let mut websocket = match response.into_websocket().await {
                Ok(ws) => ws,
                Err(e) => {
                    log::error!("WebSocket handshake failed: {}", e);
                    let _ = stream.close(Some(CloseFrame {
                        code: CloseCode::Protocol,
                        reason: format!("Handshake failed: {}", e).into(),
                    })).await;
                    return Ok(());
                }
            };
            loop {
                select! {
                    // 从客户端接收消息并转发到目标服务器
                    client_result = stream.next() => {
                        match client_result {
                            Some(Ok(rocket_ws::Message::Text(text))) => {
                                if websocket.send(reqwest_websocket::Message::Text(text)).await.is_err() {
                                    break;
                                }
                            }
                            Some(Ok(rocket_ws::Message::Binary(data))) => {
                                if websocket.send(reqwest_websocket::Message::Binary(data.into())).await.is_err() {
                                    break;
                                }
                            }
                            Some(Ok(rocket_ws::Message::Ping(data))) => {
                                if websocket.send(reqwest_websocket::Message::Ping(data.into())).await.is_err() {
                                    break;
                                }
                            }
                            Some(Ok(rocket_ws::Message::Pong(data))) => {
                                if websocket.send(reqwest_websocket::Message::Pong(data.into())).await.is_err() {
                                    break;
                                }
                            }
                            Some(Err(e)) => {
                                log::debug!("Error receiving message from client: {:?}", e);
                                break;
                            }
                            None => {
                                log::info!("Client disconnected");
                                break;
                            }
                            _ => {}
                        }
                    }
                    // 从目标服务器接收消息并转发到客户端
                    server_result = websocket.try_next() => {
                        match server_result {
                            Ok(Some(reqwest_websocket::Message::Text(text))) => {
                                if stream.send(rocket_ws::Message::Text(text)).await.is_err() {
                                    break;
                                }
                            }
                            Ok(Some(reqwest_websocket::Message::Binary(data))) => {
                                if stream.send(rocket_ws::Message::Binary(data.into())).await.is_err() {
                                    break;
                                }
                            }
                            Ok(Some(reqwest_websocket::Message::Ping(data))) => {
                                if stream.send(rocket_ws::Message::Ping(data.into())).await.is_err() {
                                    break;
                                }
                            }
                            Ok(Some(reqwest_websocket::Message::Pong(data))) => {
                                if stream.send(rocket_ws::Message::Pong(data.into())).await.is_err() {
                                    break;
                                }
                            }
                            Ok(Some(reqwest_websocket::Message::Close { .. })) => {
                                let _ = stream.close(None).await;
                                break;
                            }
                            Ok(None) => {
                                log::info!("Target server disconnected");
                                break;
                            }
                            Err(e) => {
                                log::error!("Error receiving message from target server: {:?}", e);
                                break;
                            }
                        }
                    }
                }
            }

            Ok(())
        })
    })
}
