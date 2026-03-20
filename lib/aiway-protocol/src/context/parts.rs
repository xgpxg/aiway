use http::{request, response, Uri};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SerdeParts {
    #[serde(with = "http_serde::option::method")]
    pub method: Option<http::Method>,

    #[serde(with = "http_serde::option::status_code")]
    pub status_code: Option<http::StatusCode>,

    #[serde(with = "http_serde::option::uri")]
    pub uri: Option<Uri>,

    #[serde(with = "http_serde::option::header_map")]
    pub headers: Option<http::HeaderMap>,

    #[serde(with = "http_serde::option::authority")]
    pub authority: Option<http::uri::Authority>,
}

impl From<&request::Parts> for SerdeParts {
    fn from(parts: &request::Parts) -> Self {
        Self {
            method: parts.method.clone().into(),
            status_code: None,
            uri: parts.uri.clone().into(),
            headers: parts.headers.clone().into(),
            authority: parts.uri.authority().clone().cloned(),
        }
    }
}

impl From<&response::Parts> for SerdeParts {
    fn from(parts: &response::Parts) -> Self {
        Self {
            method: None,
            status_code: parts.status.clone().into(),
            uri: None,
            headers: parts.headers.clone().into(),
            authority: None,
        }
    }
}
