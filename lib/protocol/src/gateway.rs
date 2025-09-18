//! # 网关相关协议定义
//!
//! 主要定义以下内容：
//! 1. 请求/响应上下文
//! 2. 网关与插件交互协议
//!

use crate::SV;
use dashmap::DashMap;
use serde::Deserialize;
use std::any::Any;
use std::fmt::Display;
use std::sync::Arc;

/// HTTP上下文
///
/// 包含请求上下文和响应上下文，这些内容可在请求过程中被修改。
///
/// - 内部可变性
/// 要求在实现时，不要出现对外的可变引用
/// - 该类型也作为与Plugin交互的数据结构
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
/// 该上下文不应该被多次创建，仅在请求阶段构建一次。
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
    pub query: SV<Option<String>>,
    /// 请求体
    ///
    /// 理论上，大部分请求体都是Json，可以使用serde_json序列化
    /// TODO 文件如何处理？
    pub body: SV<Vec<u8>>,
    /// 扩展数据
    pub state: DashMap<String, Box<dyn Any + Send + Sync>>,
    /// 路由信息
    ///
    /// 路由由网关根据当前请求的path匹配得到，通常情况下，路由不应该手动修改。
    /// 由于Route是网关级别的配置，对全局有效，所以使用Arc来共享
    pub route: SV<Arc<Route>>,
}

#[derive(Debug, Default)]
pub struct ResponseContext {
    /// 响应状态码
    pub status: SV<Option<u16>>,
    /// 响应头
    pub headers: DashMap<String, String>,
    /// 响应体
    pub body: SV<Vec<u8>>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Route {
    /// 名称
    name: String,
    /// 路径，支持通配符，全局唯一。
    /// 必须以"/"开头
    pub path: String,
    /// 需要路由到的服务ID
    #[serde(alias = "service_id", alias = "service-id")]
    pub service_id: String,
    /// 协议：http | sse
    protocol: String,
    /// 请求方法：get | post | put | delete | patch | options
    method: String,
    /// 前置过滤器插件，在请求阶段执行，多个按顺序串联执行
    #[serde(default = "Vec::default")]
    pre_filters: Vec<FilterPlugin>,
    /// 后置过滤器插件，在响应阶段执行，多个按顺序串联执行
    #[serde(default = "Vec::default")]
    post_filters: Vec<FilterPlugin>,
}

#[derive(Debug, Deserialize)]
pub struct FilterPlugin {
    /// 过滤器插件名称
    name: String,
    /// 阶段
    phase: String,
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
        self.query.set(Some(query.to_string()));
    }

    pub fn get_query(&self) -> Option<&str> {
        self.query
            .get()
            .and_then(|opt| opt.as_ref().map(|s| s.as_str()))
    }

    pub fn set_body(&self, body: Vec<u8>) {
        self.body.set(body)
    }

    pub fn get_body(&self) -> Option<&Vec<u8>> {
        self.body.get().clone()
    }

    pub fn set_route(&self, route: Arc<Route>) {
        self.route.set(route);
    }

    pub fn get_route(&self) -> Option<&Arc<Route>> {
        self.route.get()
    }
}

impl ResponseContext {
    pub fn set_status(&self, status: u16) {
        self.status.set(Some(status));
    }

    pub fn get_status(&self) -> Option<u16> {
        self.status.get().and_then(|opt| opt.clone())
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
}
impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Method::Get => write!(f, "GET"),
            Method::Post => write!(f, "POST"),
            Method::Put => write!(f, "PUT"),
            Method::Delete => write!(f, "DELETE"),
            Method::Head => write!(f, "HEAD"),
            Method::Options => write!(f, "OPTIONS"),
            Method::Patch => write!(f, "PATCH"),
        }
    }
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
            _ => panic!("Invalid method"),
        }
    }
}
