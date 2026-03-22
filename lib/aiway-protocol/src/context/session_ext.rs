use crate::common::constants::MODEL_API_PREFIX;
use crate::common::header::Headers;
use crate::context::request_ext::RequestExt;
use pingora_http::Method;
use pingora_proxy::Session;
use std::collections::HashMap;

pub trait SessionExt {
    fn get_request_id(&self) -> String;
    fn get_request_header(&self, key: &str) -> Option<String>;
    fn set_request_header(&mut self, key: &str, value: &str);

    fn all_request_headers(&self) -> HashMap<String, String>;
    fn get_path(&self) -> String;
    fn set_path(&mut self, path: &str);
    fn get_method(&self) -> &Method;
    fn get_host(&self) -> String;
    fn route_match_key(&self) -> String;

    fn query(&self) -> Option<String>;

    fn is_model_request(&self) -> bool;
}

impl SessionExt for Session {
    #[inline]
    fn get_request_id(&self) -> String {
        self.req_header().get_request_id()
    }

    #[inline]
    fn get_request_header(&self, key: &str) -> Option<String> {
        self.req_header().get_request_header(key)
    }

    #[inline]
    fn set_request_header(&mut self, key: &str, value: &str) {
        self.req_header_mut().set_request_header(key, value);
    }

    #[inline]
    fn all_request_headers(&self) -> HashMap<String, String> {
        self.req_header().all_request_headers()
    }

    #[inline]
    fn get_path(&self) -> String {
        self.req_header().get_path()
    }

    #[inline]
    fn set_path(&mut self, path: &str) {
        self.req_header_mut().set_path(path);
    }

    #[inline]
    fn get_method(&self) -> &Method {
        &self.req_header().method
    }

    #[inline]
    fn get_host(&self) -> String {
        self.req_header().get_host()
    }

    #[inline]
    fn route_match_key(&self) -> String {
        self.req_header().route_match_key()
    }

    #[inline]
    fn query(&self) -> Option<String> {
        self.req_header().query()
    }

    #[inline]
    fn is_model_request(&self) -> bool {
        self.get_path().starts_with(MODEL_API_PREFIX)
    }
}
