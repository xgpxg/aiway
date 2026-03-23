//use rocket::Request;

pub struct Headers;
impl Headers {
    pub const REQUEST_ID: &'static str = "x-aiway-request-id";
    pub const REQUEST_TIME: &'static str = "x-aiway-request-time";
    pub const AUTHORIZATION: &'static str = "x-aiway-authorization";
    pub const ERROR_CODE: &'static str = "x-error-code";
    pub const ERROR_MESSAGE: &'static str = "x-error-message";
    pub const REFERER: &'static str = "referer";
    pub const USER_AGENT: &'static str = "user-agent";
    pub const CONTENT_TYPE: &'static str = "content-type";
    pub const ORIGIN: &'static str = "origin";
    pub const UPGRADE: &'static str = "upgrade";
    pub const HOST: &'static str = "host";
}
