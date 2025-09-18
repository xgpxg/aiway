use crate::openapi::error::GatewayError;
use crate::openapi::response::GatewayResponse;
use rocket::{get, routes};

pub fn routes() -> Vec<rocket::Route> {
    routes![error_502]
}
#[get("/502")]
async fn error_502() -> GatewayResponse {
    GatewayResponse::Error(GatewayError::BadGateway)
}
