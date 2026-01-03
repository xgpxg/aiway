pub mod api;
#[allow(clippy::module_inception)]
mod proxy;
mod request;
mod response;

pub use proxy::Proxy;
pub use response::ModelError;
