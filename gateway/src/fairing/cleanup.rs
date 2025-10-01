//! # 清理请求上下文
//!
//! 最后执行，清理RequestContext
//!
use crate::context;
use crate::context::HCM;
use crate::report::STATE;
use rocket::Request;
use rocket::fairing::Fairing;

pub struct Cleaner {}
impl Cleaner {
    pub fn new() -> Self {
        Self {}
    }
}

#[rocket::async_trait]
impl Fairing for Cleaner {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "Cleaner",
            kind: rocket::fairing::Kind::Response,
        }
    }

    async fn on_response<'r>(&self, req: &'r Request<'_>, res: &mut rocket::Response<'r>) {
        if let Some(request_id) = req.headers().get_one(context::Headers::REQUEST_ID) {
            HCM.remove(request_id);
        }

        res.remove_header(context::Headers::ERROR_CODE);
        res.remove_header(context::Headers::ERROR_MESSAGE);

        STATE.inc_http_connect_count(-1);
    }
}
