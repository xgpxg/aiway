//! # 控制台服务
//! 控制台主要面向使用者，提供网关管理、监控、通知等功能。
//!
//! # 功能模块
//! # 仪表盘
//! 总览整个系统状态。指标待定。
//!
//! ## 网关
//!
//! ### 路由配置
//! 路由增删改查。
//!
//! 主要配置项：
//! - 路由基本信息
//! - 路径
//! - host（待定）
//! - 目标服务，仅支持一个
//!
//! 路由配置变更后重新构建路由表，推送到conreg，网关在监听到路由表变化时，会重新加载。
//!
//! 注意：目前conreg-client还不支持修改配置，需要更新一下才能满足要求。
//!
//! ### 服务管理
//! 服务增删改查。
//!
//! 主要配置项：
//! - 服务基本信息
//! - 服务节点，可多个
//! - 负载策略
//! - 降级策略（待确认）
//!
//! 同样的，服务变更后，重新构建服务列表，推到conreg，网关在监听到服务列表变化时重新加载。
//!
//! ### 插件管理
//! 插件增删改查。目前插件仅支持系统预定义的，后续支持以脚本的方式自定义插件。
//!
//! 主要配置项：
//! - 插件基本信息
//! - 下载地址
//! - TODO 插件本身的配置如何做？
//!
//! 插件（这里指全局插件）变更后，推送到conreg，网关在监听到插件列表变化时重新加载。
//!
//! ### 日志
//! 日志查询、分析。
//!
//! 可按时间范围、关键字、日志级别查询。
//!
//! ## 安全
//! ### 密钥管理
//! 密钥管理原计划使用一个单独的KMS服务，但后来发现并没有必要，且鉴权 是个频繁操作，如果使用单独服务会额外增加一次远程调用。
//! 所以将验证操作放在了网关内部。
//!
//! 控制台变更了密钥后，同步到Redis，网关从Redis中读取。
//!
//! ### 安全监控
//! 主要监控网络请求以及分析日志，发现异常流量或者日志里发现异常发送警告通知。
//!
//! ## 系统
//! ### 用户管理
//! 提供一个简单的用户管理功能，可以查询用户列表、新建和删除用户。
//!
//! ### 操作日志
//! 主要记录控制台内部操作日志，直接存储在Mysql，注意不要和日志服务混淆。
//!
//! ### 系统设置
//! 系统级配置，配置项待定。
//!
//! # 系统交互
//! 【控制台】 调用 【Mysql】【Redis】【Conreg】
//!
//! 【控制台】 监听 【message】
//!
//! 【网关】 调用 【Conreg】【Redis】
//!
//! 【网关】 监听 【Conreg】

use crate::config::config::AppConfig;
use rocket::data::{ByteUnit, Limits};
use rocket::{Config, routes};
use std::net::IpAddr;
use std::str::FromStr;
mod auth;
mod common;
pub mod db;
mod gateway;
mod key;
mod route;
mod service;
mod user;

pub async fn start_http_server() -> anyhow::Result<()> {
    let config = &AppConfig::server();
    let mut builder = rocket::build().configure(Config {
        address: IpAddr::from_str(config.address.as_str())?,
        port: config.port,
        limits: Limits::default()
            .limit("json", ByteUnit::Mebibyte(3))
            .limit("data-form", ByteUnit::Mebibyte(100))
            .limit("file", ByteUnit::Mebibyte(100)),
        log_level: rocket::config::LogLevel::Critical,
        cli_colors: false,
        ..Config::debug_default()
    });

    builder = builder.mount("/api/v1", gateway::api::routes());

    builder = builder.mount("/api/user", user::api::routes());
    builder = builder.mount("/api/route", route::api::routes());
    builder = builder.mount("/api/service", service::api::routes());
    builder = builder.mount("/api/key", key::api::routes());

    builder.launch().await?;

    Ok(())
}
