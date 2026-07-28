use thiserror::Error;

#[derive(Error, Debug)]
pub enum CertError {
    #[error("ACME error: {0}")]
    Acme(#[from] acme2::Error),

    #[error("DNS provider {provider} error: {detail}")]
    DnsProvider {
        provider: &'static str,
        detail: String,
    },

    #[error("DNS propagation timeout for {domain}")]
    DnsPropagationTimeout { domain: String },

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("OpenSSL error: {0}")]
    OpenSsl(#[from] openssl::error::ErrorStack),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}
