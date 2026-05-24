use crate::server::db::Pool;
use crate::server::db::models::domain::{Domain, DomainStatus, Protocol};
use aiway_protocol::gateway::CertEntry;
use rbs::value;

/// 查询所有启用的 HTTPS 域名证书，返回给网关用于 SNI 动态匹配
pub(crate) async fn domains() -> anyhow::Result<Vec<CertEntry>> {
    let domains = Domain::select_by_map(
        Pool::get()?,
        value! {"status": DomainStatus::Ok, "protocol": Protocol::HTTPS},
    )
    .await?;
    let mut list = Vec::with_capacity(domains.len());
    for d in domains {
        let domain = match d.domain {
            Some(d) if !d.is_empty() => d,
            _ => continue,
        };
        list.push(CertEntry {
            domain,
            cert: d.cert.unwrap_or_default().into_bytes(),
            key: d.cert_key.unwrap_or_default().into_bytes(),
        });
    }
    Ok(list)
}
