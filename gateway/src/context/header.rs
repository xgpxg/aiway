use rocket::Request;

pub struct Headers;
impl Headers {
    pub const REQUEST_ID: &'static str = "X-Aiway-Request-Id";
    pub const REQUEST_TIME: &'static str = "X-Aiway-Request-Time";
    pub const AUTHORIZATION: &'static str = "Authorization";
    pub const ERROR_CODE: &'static str = "X-Error-Code";
    pub const ERROR_MESSAGE: &'static str = "X-Error-Message";
    pub const REFERER: &'static str = "Referer";
    pub const USER_AGENT: &'static str = "User-Agent";
}

impl Headers {
    pub fn get_request_id(req: &Request) -> String {
        req.headers()
            .get_one(Headers::REQUEST_ID)
            .unwrap()
            .to_string()
    }
}
