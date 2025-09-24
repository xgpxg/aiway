//! # 日志记录
//!
//! 在请求结束前记录日志，理论上该fairing总是需要被调用

use crate::context::HCM;
use rocket::Request;
use rocket::fairing::Fairing;

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

    async fn on_response<'r>(&self, req: &'r Request<'_>, _res: &mut rocket::Response<'r>) {
        // 提取RequestContext
        let _context = HCM.get_from_request(&req);

        // 记录日志


        //println!("In logger: {:?}", context);
        //println!("Run Logger on response");
    }
}
