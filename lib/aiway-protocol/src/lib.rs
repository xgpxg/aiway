//! # aiway交互协议
//!
#[allow(unused)]
pub mod common;
pub mod context;
#[allow(unused)]
pub mod gateway;
#[cfg(feature = "logg")]
pub mod logg;
#[cfg(feature = "mcp")]
pub mod mcp;
#[cfg(feature = "model")]
pub mod model;

#[allow(unused)]
mod single;

#[allow(unused)]
pub use single::SingleValue as SV;

#[cfg(feature = "mcp")]
pub use rmcp;
