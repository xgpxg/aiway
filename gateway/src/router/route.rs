#[derive(Debug, Default)]
pub struct Route {
    /// 名称
    name: String,
    /// 路径，支持通配符，全局唯一
    pub path: String,
    /// 需要路由到的服务ID
    pub service_id: String,
    /// 协议：http | sse
    protocol: String,
    /// 请求方法：get | post | put | delete | patch | options
    method: String,
    /// 前置过滤器插件，在请求阶段执行，多个按顺序串联执行
    pre_filters: Vec<FilterPlugin>,
    /// 后置过滤器插件，在响应阶段执行，多个按顺序串联执行
    post_filters: Vec<FilterPlugin>,
}

#[derive(Debug)]
pub struct FilterPlugin {
    /// 过滤器插件名称
    name: String,
    /// 阶段
    phase: String,
}
