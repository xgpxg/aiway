mod client;
#[allow(clippy::module_inception)]
mod proxy;

pub use proxy::Proxy;



#[derive(thiserror::Error, Debug)]
pub enum ModelError {
    /// 请求模型提供商时发生的错误，如API地址错误，连接失败，服务器无响应等无法连接的情况
    /// 该错误不会进入插件处理
    /// 响应状态码：500
    #[error("{0}")]
    RequestProviderError(String),
    // /// 调用模型提供商API时的错误，该错误会进入插件处理
    // #[error("{0} {1}")]
    // ApiError(u16, String),
    /// 不支持的模型错误，响应状态码：400
    #[error("Unsupported model: {0}")]
    UnsupportedModel(String),
    /// 没有可用的提供商，响应状态码：500
    #[error("No available provider")]
    NoAvailableProvider,
    /// 解析错误，响应状态码：500
    #[error("Parse error")]
    Parse(String),
    // /// 插件执行错误，响应状态码：500
    // #[error("Plugin error: {0}")]
    // PluginError(String),
    // /// 未知错误，响应状态码：500
    // #[error("Unknown error: {0}")]
    // Unknown(String),
}

impl ModelError {
    pub fn into_status_message(self) -> (u16, String) {
        match self {
            ModelError::RequestProviderError(_) => (500, "请求模型提供商时发生错误".to_string()),
            //ModelError::ApiError(status_code, message) => (status_code, message),
            ModelError::UnsupportedModel(_) => (400, "不支持的模型".to_string()),
            ModelError::NoAvailableProvider => (500, "没有可用的提供商".to_string()),
            ModelError::Parse(message) => (500, message),
            //ModelError::PluginError(message) => (500, message),
            //ModelError::Unknown(message) => (500, message),
        }
    }
}