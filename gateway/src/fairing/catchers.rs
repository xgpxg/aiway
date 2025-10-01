//! # Catchers
//! - 在response fairing之前执行
//! - 只会处理网关内部异常，下游服务的异常不会被处理，而是透传

use crate::extract_error;
use rocket::Request;

#[rocket::catch(401)]
pub fn catch_401(req: &Request) -> String {
    if let Some((_, message)) = extract_error!(req) {
        return message.to_string();
    }
    "Unauthorized".to_string()
}

#[rocket::catch(403)]
pub fn catch_403(req: &Request) -> String {
    if let Some((_, message)) = extract_error!(req) {
        return message.to_string();
    }
    "Forbidden".to_string()
}

#[rocket::catch(404)]
pub fn catch_404(req: &Request) -> String {
    if let Some((_, message)) = extract_error!(req) {
        return message.to_string();
    }
    "NotFound".to_string()
}
