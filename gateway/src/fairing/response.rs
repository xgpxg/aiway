//! # 设置响应数据
//! ## 主要功能
//! 在响应客户端前，将上下文中的响应数据附加到响应中。
//!
//! ## 基本准则
//! - 该fairing必须执行
//! - 使用覆盖模式，即上下文中的响应数据优先覆盖原始响应中的数据。这是因为，上下文中的数据可能是由插件修改而来，应该优先被设置。
//!
use crate::context::{HCM, Headers};
use crate::report::STATE;
use rocket::Request;
use rocket::fairing::Fairing;
use rocket::http::{Header, Status};
use std::io::Cursor;

pub struct ResponseData {}
impl ResponseData {
    pub fn new() -> Self {
        Self {}
    }
}

#[rocket::async_trait]
impl Fairing for ResponseData {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "ResponseData",
            kind: rocket::fairing::Kind::Response,
        }
    }

    async fn on_response<'r>(&self, req: &'r Request<'_>, res: &mut rocket::Response<'r>) {
        let request_context = &HCM.get_from_request(&req).request;
        let response_context = &HCM.get_from_request(&req).response;
        response_context.set_response_ts(chrono::Local::now().timestamp_millis());
        // 设置状态码
        if let Some(status) = response_context.status.get() {
            if let Some(status) = status {
                res.set_status(Status::new(*status));
            }
        }

        response_context.headers.iter().for_each(|header| {
            res.set_header(Header::new(header.key().clone(), header.value().clone()));
        });

        if let Some(body) = response_context.body.get() {
            if body.len() > 0 {
                res.set_sized_body(body.len(), Cursor::new(body.clone()));
            }
        }

        // 添加请求ID
        res.set_header(Header::new(
            Headers::REQUEST_ID,
            Headers::get_request_id(req),
        ));

        STATE.inc_status_request_count(res.status().code, 1);
        STATE.inc_response_time(
            (response_context.get_response_ts() - request_context.get_request_ts()) as usize,
        );
        //println!("ResponseData: {:?}", res);
    }
}
