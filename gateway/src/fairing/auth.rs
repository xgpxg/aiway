//! # 鉴权
//! ## 主要功能
//! 从请求中提取ApiKey并验证，验证不通过则返回403。
//! 验证通过后
//!
//! ## 基本准则
//! - 在基本安全验证后执行。
//! - 由系统内置，不可关闭。
//! - 不应提取请求body数据，仅对请求url（含query参数）、header等基础数据进行验证。
//! - 当验证失败时，更改uri到指定端点，返回错误信息。
//! - 不应涉及任何网络请求及IO操作，需要在5ms内完成
//!
use crate::context::Headers;
use rocket::fairing::Fairing;
use rocket::http::uri::Origin;
use rocket::http::Method;
use rocket::{Data, Request};

pub struct Authentication {}
impl Authentication {
    pub fn new() -> Self {
        Self {}
    }
}

const BEARER_PREFIX: &str = "Bearer ";

#[rocket::async_trait]
impl Fairing for Authentication {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "Authentication",
            kind: rocket::fairing::Kind::Request,
        }
    }

    async fn on_request(&self, req: &mut Request<'_>, _data: &mut Data<'_>) {
        let _ = crate::extract_api_path!(req);

        let api_key = req.headers().get_one(Headers::AUTHORIZATION);
        let api_key = match api_key {
            Some(api_key) => match api_key.strip_prefix(BEARER_PREFIX) {
                Some(api_key) => api_key,
                None => {
                    req.set_method(Method::Get);
                    req.set_uri(Origin::parse("/eep/401").unwrap());
                    return;
                }
            },
            None => {
                req.set_method(Method::Get);
                req.set_uri(Origin::parse("/eep/401").unwrap());
                return;
            }
        };

        // TODO 调用KMS鉴权
    }
}
