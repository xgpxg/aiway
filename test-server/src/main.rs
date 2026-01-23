use rocket::http::Status;
use rocket::response::stream::{Event, EventStream};
use rocket::{Config, get, post, routes};
use std::net::IpAddr;
use std::str::FromStr;

/// 测试用
#[rocket::main]
async fn main() -> anyhow::Result<()> {
    let mut builder = rocket::build().configure(Config {
        address: IpAddr::from_str("0.0.0.0")?,
        port: 8080,
        log_level: rocket::config::LogLevel::Critical,
        ..Config::debug_default()
    });

    builder = builder.mount("/", routes![hello, hello_post, sse, html, ws]);

    builder.launch().await?;

    Ok(())
}

#[get("/hello")]
fn hello() -> &'static str {
    "World"
}

#[post("/hello")]
fn hello_post() -> &'static str {
    "World"
}

#[get("/sse")]
fn sse() -> EventStream![] {
    EventStream! {
        for _ in 0..10 {
            yield Event::data("ping");
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }
}

#[get("/ws")]
fn ws(ws: rocket_ws::WebSocket) -> rocket_ws::Channel<'static> {
    use rocket::futures::{SinkExt, StreamExt};

    ws.channel(move |mut stream| {
        Box::pin(async move {
            while let Some(message) = stream.next().await {
                let _ = stream.send(message?).await;
            }

            Ok(())
        })
    })
}

#[get("/html")]
fn html() -> String {
    include_str!("index.html").to_string()
}
