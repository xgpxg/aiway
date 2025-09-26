//! # 本地共享缓存
//! 用于多进程共享缓存，基于zbus实现
//!
//! 场景：
//! 单机模式下，控制台和网关通过子进程的方式集成在一起，需要同时访问本地缓存，
//! 而目前使用的sled仅支持单进程访问，因此需要适配为支持多进程访问的。
//! 而多进程通信通常使用IPC实现，所以实现为，在单节点的主进程启动时，
//! 初始化sled，并暴露出一个service，子进程通过IPC的方式访问该service，实现多进程共享的缓存。
mod client;
mod server;
pub use client::ShareCache;
pub use server::start_share_cache_server;
