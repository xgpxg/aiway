use rocket::Request;

pub struct Headers;
impl Headers {
    pub const REQUEST_ID: &'static str = "X-Aiway-Request-Id";
}

impl Headers {
    pub fn get_request_id(req: &Request) -> String {
        req.headers()
            .get_one(Headers::REQUEST_ID)
            .unwrap()
            .to_string()
    }
}
