use busi::impl_pagination;
use busi::req::PageReq;
use rocket::serde::{Deserialize, Serialize};

/// 签发证书请求
///
/// 邮箱、DNS 提供商及凭证从系统配置（AcmeConfig）读取，无需每次传入。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertIssueReq {
    /// 域名，如 example.com 或 *.example.com
    pub domain: String,
    /// 是否自动续期
    #[serde(default)]
    pub auto_renew: bool,
    /// 备注
    pub remark: Option<String>,
}

/// 证书列表查询请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertListReq {
    pub page: PageReq,
    pub filter_text: Option<String>,
    pub status: Option<String>,
}
impl_pagination!(CertListReq);

/// 设置自动续期
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetAutoRenewReq {
    pub id: i64,
    pub auto_renew: bool,
}
