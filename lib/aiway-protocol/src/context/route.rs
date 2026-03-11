use crate::gateway::plugin::ConfiguredPlugin;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Route {
    /// 路由名称
    pub name: String,
    /// Host，可选，`*` 代表不限制。
    /// 支持泛域名，格式为 `*.example.com`，通配符只能出现在域名开头。
    pub host: String,
    /// 路径，支持通配符，必须以 `/` 开头，全局唯一。
    /// 同一域名下的路径不能存在冲突，如 `/api/*` 和 `/api/**` 就是冲突的路径，在控制台保存时需要校验。
    /// 所有为 `*` 的域名下的路径也不能冲突。
    pub path: String,
    /// 转换后的匹配路径。
    /// 由于路径匹配组件使用的是`matchit`，格式为`/xxx/{*any}`，
    /// 它不符合常用的`/api/**`的格式，而`path`储存的是常用格式，
    /// 所以这里需要做一次转换。
    pub match_paths: Vec<String>,
    /// 需要路由到的服务ID，例如 `user-service`
    pub service: String,
    /// 请求方法：GET | POST | PUT | DELETE | PATCH | OPTIONS | HEAD
    /// 支持配置多个。当为空数组时表示不限制。
    /// 不参与路由唯一性验证。
    pub methods: Vec<String>,
    /// header匹配条件。
    /// 当存在多个时，所有条件都匹配时才算匹配成功。
    /// 不参与路由唯一性验证。
    #[serde(alias = "header_condition", alias = "header-condition")]
    pub header: BTreeMap<String, String>,
    /// query匹配条件
    /// 当存在多个时，所有条件都匹配时才算匹配成功。
    /// 不参与路由唯一性验证。
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
    /// 是否开启鉴权。
    /// 启用后，需要在控制台的“密钥管理”处创建API Key，
    /// 并通过请求头 `X-Aiway-Authorization` 传入，作为网关的鉴权凭证。
    #[serde(default = "bool::default", alias = "is_auth", alias = "is-auth")]
    pub is_auth: bool,
    /// 鉴权路径白名单
    pub auth_white_list: Vec<String>,
}

impl Route {
    pub fn get_service(&self) -> &String {
        &self.service
    }

    /// host + path作为匹配的key
    pub fn to_match_keys(&self) -> Vec<String> {
        self.match_paths
            .iter()
            .map(|p| {
                if self.host == "*" {
                    format!("{{host}}{}", p)
                } else {
                    format!("{}{}", self.host, p)
                }
            })
            .collect()
    }
}
