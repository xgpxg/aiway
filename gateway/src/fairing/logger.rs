use rocket::fairing::Fairing;
use rocket::{Data, Request};

/// openapi鉴权
pub struct Logger {}
impl Logger {
    pub fn new() -> Self {
        Self {}
    }
}

#[rocket::async_trait]
impl Fairing for Logger {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "Logger",
            kind: rocket::fairing::Kind::Request | rocket::fairing::Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _req: &'r Request<'_>, _res: &mut rocket::Response<'r>) {
        // 提取RequestContext

        //println!("Run Logger on response");
    }
}
