use rocket::Request;
use rocket::response::Responder;

pub enum GatewayError {
    NotFound,
}

impl<'r> Responder<'r, 'r> for GatewayError {
    fn respond_to(self, _request: &'r Request<'_>) -> rocket::response::Result<'r> {
        match self {
            GatewayError::NotFound => rocket::response::Response::build()
                .status(rocket::http::Status::NotFound)
                .ok(),
        }
    }
}
