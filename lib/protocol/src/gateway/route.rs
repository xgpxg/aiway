use crate::SV;
use dashmap::DashMap;
use serde::Deserialize;
use std::str::FromStr;

#[derive(Debug, Default, Deserialize)]
pub struct Route {
    /// 名称
    pub name: String,
    // 域名
    // 暂不实现域名匹配，由Nginx处理
    //pub host: String,
    /// 前缀，必须以"/"开头，全局唯一。
    pub prefix: Option<String>,
    /// 路径，支持通配符，必须以"/"开头，全局唯一。
    pub path: String,
    /// 是否去除路径前缀，默认为false
    #[serde(
        default = "bool::default",
        alias = "strip_prefix",
        alias = "strip-prefix"
    )]
    pub strip_prefix: bool,
    /// 需要路由到的服务ID
    pub service: SV<String>,
    /// 协议：http | sse
    pub protocol: String,
    /// 请求方法：get | post | put | delete | patch | options
    pub method: String,
    /// header匹配条件
    #[serde(alias = "header_condition", alias = "header-condition")]
    pub header: Option<DashMap<String, String>>,
    /// query匹配条件，满足
    #[serde(alias = "query_condition", alias = "query-condition")]
    pub query: Option<DashMap<String, String>>,
    /// 前置过滤器插件，在请求阶段执行，多个按顺序串联执行
    #[serde(default = "Vec::default", alias = "pre_filters", alias = "pre-filters")]
    pre_filters: Vec<FilterPlugin>,
    /// 后置过滤器插件，在响应阶段执行，多个按顺序串联执行
    #[serde(
        default = "Vec::default",
        alias = "post_filters",
        alias = "post-filters"
    )]
    post_filters: Vec<FilterPlugin>,
}

#[derive(Debug, Deserialize)]
pub struct FilterPlugin {
    /// 过滤器插件名称
    name: String,
    /// 阶段
    phase: String,
}

impl Route {
    pub fn get_service(&self) -> Option<&String> {
        self.service.get()
    }

    /// 构建请求路径
    ///
    /// - path 原始的请求路径
    ///
    /// 能执行到这里，说明已经匹配到该路由了。
    /// 根据`strip_prefix`决定是否移除前缀
    pub fn build_path(&self, path: &str) -> String {
        if self.strip_prefix {
            if let Some(prefix) = &self.prefix {
                path[prefix.len() - 1..].to_string()
            } else {
                path.to_string()
            }
        } else {
            path.to_string()
        }
    }
}
