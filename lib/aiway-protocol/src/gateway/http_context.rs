use crate::gateway::request_context::RequestContext;
use crate::gateway::response_context::ResponseContext;

/// HTTP上下文
///
/// 包含请求上下文和响应上下文，这些内容可在请求过程中被修改。
///
/// - 内部可变性
///   要求在实现时，不要出现对外的可变引用
/// - 该类型也作为与Plugin交互的数据结构
#[derive(Debug, Default)]
pub struct HttpContext {
    /// 请求上下文，应该在请求阶段构建
    pub request: RequestContext,
    /// 响应上下文，在构建请求上下文时同步构建，在响应阶段更新
    pub response: ResponseContext,
}
