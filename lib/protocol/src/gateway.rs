//! # 网关相关协议定义
//!
//! 主要定义以下内容：
//! 1. 请求上下文
//! 2. 网关与插件交互协议
//!

use crate::SV;
use dashmap::DashMap;
use std::any::Any;

/// HTTP上下文
///
/// 包含请求上下文和响应上下文，这些内容可在请求过程中被修改。
///
/// - 内部可变性
/// 要求在实现时，不要出现对外的可变引用
#[derive(Debug)]
pub struct HttpContext {
    /// 请求上下文，应该在请求阶段构建
    pub request: RequestContext,
    /// 响应上下文，在构建请求上下文时同步构建，在响应阶段更新
    pub response: ResponseContext,
}

/// 请求上下文
///
/// 该上下文贯穿整个请求流程，同一个请求中应该仅持有一份数据。
///
#[derive(Debug, Default)]
pub struct RequestContext {
    /// 请求ID
    pub request_id: String,
    /// 请求方法
    pub method: SV<Method>,
    /// 请求头
    pub headers: DashMap<String, String>,
    /// 请求路径。
    pub path: SV<String>,
    /// 请求参数
    pub query: SV<String>,
    /// 请求体
    pub body: SV<Vec<u8>>,
    /// 扩展数据
    pub state: DashMap<String, Box<dyn Any + Send + Sync>>,
}

#[derive(Debug, Default)]
pub struct ResponseContext {
    pub status: SV<u16>,
    pub headers: DashMap<String, String>,
    pub body: SV<Vec<u8>>,
}

impl RequestContext {
    pub fn set_path(&self, path: &str) {
        self.path.set(path.to_string());
    }

    pub fn get_path(&self) -> String {
        self.path.get().cloned().unwrap_or_default()
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
        self.query.set(query.to_string());
    }

    pub fn get_query(&self) -> String {
        self.query.get().cloned().unwrap_or_default()
    }
}

impl ResponseContext {
    pub fn set_status(&self, status: u16) {
        self.status.set(status);
    }
    pub fn get_status(&self) -> u16 {
        self.status.get().cloned().unwrap_or_default()
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

    pub fn set_body(&self, body: Vec<u8>) {
        self.body.set(body)
    }

    pub fn get_body(&self) -> Option<&Vec<u8>> {
        self.body.get().clone()
    }
}

#[derive(Debug, Default)]
pub enum Method {
    #[default]
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
