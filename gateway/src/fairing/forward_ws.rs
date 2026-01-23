use rocket::fairing::Fairing;
use rocket::{Data, Request};
use context::skip_if_error;

pub struct ForwardWebsocketFairing {}
impl ForwardWebsocketFairing {
    pub fn new() -> Self {
        Self {}
    }
}

#[rocket::async_trait]
impl Fairing for ForwardWebsocketFairing {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "ForwardWebsocket",
            kind: rocket::fairing::Kind::Request,
        }
    }

    async fn on_request(&self, req: &mut Request<'_>, _data: &mut Data<'_>) {
        skip_if_error!(req);
        // 判断是否是websocket
        if req.headers().get_one("Upgrade") == Some("websocket") {
            // 转发到ws端点
            let new_uri = req.uri().map_path(|p| format!("/ws{}", p)).unwrap();
            req.set_uri(new_uri);
        }
    }
}
