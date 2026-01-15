use rocket::Request;

pub struct Headers;
impl Headers {
    pub const REQUEST_ID: &'static str = "x-aiway-request-id";
    pub const REQUEST_TIME: &'static str = "x-aiway-request-time";
    pub const AUTHORIZATION: &'static str = "authorization";
    pub const ERROR_CODE: &'static str = "x-error-code";
    pub const ERROR_MESSAGE: &'static str = "x-error-message";
    pub const REFERER: &'static str = "referer";
    pub const USER_AGENT: &'static str = "user-agent";
    pub const CONTENT_TYPE: &'static str = "content-type";
}

impl Headers {
    pub fn get_request_id(req: &Request) -> String {
        req.headers()
            .get_one(Headers::REQUEST_ID)
            .unwrap()
            .to_string()
    }
}
