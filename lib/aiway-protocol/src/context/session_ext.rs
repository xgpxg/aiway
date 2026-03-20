use pingora_http::Method;
use pingora_proxy::Session;
use std::collections::HashMap;

pub trait SessionExt {
    fn get_request_header(&self, key: &str) -> Option<String>;
    fn set_request_header(&mut self, key: &str, value: &str);

    fn all_request_headers(&self) -> HashMap<String, String>;
    fn get_path(&self) -> String;
    fn set_path(&mut self, path: &str);
    fn get_method(&self) -> &Method;
    fn get_host(&self) -> String;
    fn route_match_key(&self) -> String;

    fn query(&self) -> Option<String>;
}

impl SessionExt for Session {
    fn get_request_header(&self, key: &str) -> Option<String> {
        self.get_header(key)
            .map(|s| s.to_str().unwrap().to_string())
    }

    fn set_request_header(&mut self, key: &str, value: &str) {
        let _ = self
            .req_header_mut()
            .insert_header(key.to_string(), value.to_string());
    }

    fn all_request_headers(&self) -> HashMap<String, String> {
        self.req_header()
            .headers
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap().to_string()))
            .collect()
    }

    fn get_path(&self) -> String {
        self.req_header().uri.path().to_string()
    }

    fn set_path(&mut self, path: &str) {
        let old_uri = self.req_header().uri.clone();
        let mut parts = old_uri.into_parts();

        // 构建新的 path_and_query
        let new_path = if let Some(pq) = parts.path_and_query {
            match pq.query() {
                Some(query) => format!("{}?{}", path, query),
                None => path.to_string(),
            }
        } else {
            path.to_string()
        };

        parts.path_and_query = Some(new_path.parse().unwrap());

        if let Ok(new_uri) = http::Uri::from_parts(parts) {
            self.req_header_mut().uri = new_uri;
        }
    }

    fn get_method(&self) -> &Method {
        &self.req_header().method
    }

    fn get_host(&self) -> String {
        if self.is_http2() {
            self.get_request_header(":authority").unwrap()
        } else {
            self.get_request_header("host").unwrap()
        }
    }

    fn route_match_key(&self) -> String {
        format!("{}{}", self.get_host(), self.get_path())
    }

    fn query(&self) -> Option<String> {
        self.req_header().uri.query().map(|s| s.to_string())
    }
}
