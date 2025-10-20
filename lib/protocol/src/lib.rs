//! # 内部交互协议定义
//!
//! 这个lib仅在内部使用，不用作对外的SDK。对外SDK使用单独仓库。
//!

#[allow(unused)]
pub mod common;
#[allow(unused)]
pub mod gateway;
#[cfg(feature = "logg")]
pub mod logg;
mod single;

pub use single::SingleValue as SV;
