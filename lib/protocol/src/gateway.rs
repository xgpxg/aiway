//! # 网关相关协议定义
//!
//! 主要定义以下内容：
//! 1. 请求上下文
//! 2. 网关与插件交互协议
//!

use dashmap::{DashMap, DashSet};

/// 请求上下文
///
/// 该上下文贯穿整个请求流程，同一个请求中应该仅持有一份数据。
///
/// 锁消除：
/// 使用单个元素的DashSet来通过不可变引用获取内部的可变性，消除锁
///
#[derive(Debug)]
pub struct RequestContext {
    /// 请求ID
    pub request_id: String,
    pub method: Method,
    /// 请求头
    pub headers: DashMap<String, String>,
    /// 请求路径。
    pub path: DashSet<String>,
    /// 请求参数
    pub query: DashSet<String>,
    /// 请求体
    pub body: DashSet<Vec<u8>>,
}
impl RequestContext {
    pub fn set_path(&self, path: &str) {
        self.path.clear();
        self.path.insert(path.to_string());
    }

    pub fn get_path(&self) -> String {
        self.path
            .iter()
            .next()
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    pub fn set_header(&self, key: &str, value: &str) {
        self.headers.insert(key.to_string(), value.to_string());
    }
    pub fn get_header(&self, key: &str) -> Option<String> {
        self.headers.get(key).map(|v| v.value().clone())
    }
    pub fn remove_header(&self, key: &str) {
        self.headers.remove(key);
    }

    pub fn set_query(&self, query: &str) {
        self.query.clear();
        self.query.insert(query.to_string());
    }

    pub fn get_query(&self) -> String {
        self.query
            .iter()
            .next()
            .map(|v| v.clone())
            .unwrap_or_default()
    }
}

#[derive(Debug)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Patch,
    Connect,
    Trace,
}

impl From<&str> for Method {
    fn from(s: &str) -> Self {
        match s {
            "GET" | "get" => Method::Get,
            "POST" | "post" => Method::Post,
            "PUT" | "put" => Method::Put,
            "DELETE" | "delete" => Method::Delete,
            "HEAD" | "head" => Method::Head,
            "OPTIONS" | "options" => Method::Options,
            "PATCH" | "patch" => Method::Patch,
            "CONNECT" | "connect" => Method::Connect,
            "TRACE" | "trace" => Method::Trace,
            _ => panic!("Invalid method"),
        }
    }
}
