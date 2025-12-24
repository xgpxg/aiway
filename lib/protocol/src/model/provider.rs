use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Provider {
    /// 提供商名称，全局唯一
    pub name: String,
    /// 提供商 API 地址
    pub api_url: String,
    /// 提供商 API 密钥
    ///
    /// 如果是本地模型，可能不需要密钥，所以密钥是可选的
    pub api_key: Option<String>,
    /// 权重，默认为 1
    pub weight: u32,
}
