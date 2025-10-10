use crate::gateway::plugin::ConfiguredPlugin;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::str::FromStr;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Route {
    /// 名称
    pub name: String,
    /// Host
    /// TODO 必须的还是可选呢？？
    pub host: Option<String>,
    // 前缀，必须以"/"开头，全局唯一。
    //pub prefix: Option<String>,
    /// 路径，支持通配符，必须以"/"开头，全局唯一。
    pub path: String,
    // 是否去除路径前缀，默认为false
    // #[serde(
    //     default = "bool::default",
    //     alias = "strip_prefix",
    //     alias = "strip-prefix"
    // )]
    // pub strip_prefix: bool,
    /// 需要路由到的服务ID
    pub service: String,
    /*/// 协议：http | sse
    pub protocol: String,
    /// 请求方法：get | post | put | delete | patch | options
    pub method: String,*/
    /// header匹配条件
    #[serde(alias = "header_condition", alias = "header-condition")]
    pub header: BTreeMap<String, String>,
    /// query匹配条件，满足
    #[serde(alias = "query_condition", alias = "query-condition")]
    pub query: BTreeMap<String, String>,
    /// 前置过滤器插件，在请求阶段执行，多个按顺序串联执行
    #[serde(default = "Vec::default", alias = "pre_filters", alias = "pre-filters")]
    pub pre_filters: Vec<ConfiguredPlugin>,
    /// 后置过滤器插件，在响应阶段执行，多个按顺序串联执行
    #[serde(
        default = "Vec::default",
        alias = "post_filters",
        alias = "post-filters"
    )]
    pub post_filters: Vec<ConfiguredPlugin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewriteRule {
    /// 匹配模式（正则表达式）
    pub pattern: String,
    /// 替换字符串
    pub replacement: String,
}
impl Route {
    pub fn get_service(&self) -> &String {
        &self.service
    }

    /// 构建请求路径
    ///
    /// - path 原始的请求路径
    ///
    /// 能执行到这里，说明已经匹配到该路由了。
    /// (废弃，由插件实现)根据`strip_prefix`决定是否移除前缀
    pub fn build_path(&self, path: &str) -> String {
        /*let rewritten_path = self.apply_path_rewrite(path);*/
        path.to_string()
    }

    /*pub fn apply_path_rewrite(&self, path: &str) -> String {
        if let Some(rules) = &self.rewrite_rules {
            let mut result = path.to_string();
            for rule in rules {
                if let Ok(regex) = regex::Regex::new(&rule.pattern) {
                    result = regex.replace_all(&result, &rule.replacement).to_string();
                }
            }
            result
        } else {
            path.to_string()
        }
    }*/
}
