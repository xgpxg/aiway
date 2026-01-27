//! # aiway交互协议
//!
#[allow(unused)]
pub mod common;
pub mod context;
#[allow(unused)]
pub mod gateway;
#[cfg(feature = "logg")]
pub mod logg;
#[cfg(feature = "model")]
pub mod model;
mod single;

pub use single::SingleValue as SV;
