use crate::server::auth::UserPrincipal;
use crate::server::cert::CertListReq;
use crate::server::cert::request::{CertIssueReq, SetAutoRenewReq};
use crate::server::cert::response::{CertDetailRes, CertKeyRes, CertListRes};
use crate::server::db::models::domain::{Domain, DomainBuilder};
use crate::server::db::models::tls_cert::{TlsCert, TlsCertBuilder};
use crate::server::db::{Pool, tools};
use crate::server::system::acme::service as acme_service;
use crate::server::system::acme::{AcmeConfig, DnsCredentials};
use anyhow::{Context, bail};
use busi::req::{IdReq, IdsReq, Pagination};
use busi::res::{IntoPageRes, PageRes};
use certs::{CertIssuer, DnsProvider};
use common::id;
use logging::log;
use rbs::value;

/// 签发证书
pub async fn issue(req: CertIssueReq, user: UserPrincipal) -> anyhow::Result<i64> {
    // 检查域名是否已存在证书
    let existing = TlsCert::select_by_map(
        Pool::get()?,
        value! {
            "domain": &req.domain
        },
    )
    .await?;
    if !existing.is_empty() {
        bail!("Certificate already exists for domain: {}", req.domain);
    }

    let acme_config = acme_service::get_raw().await?;

    if acme_config.dns_provider.is_empty() {
        bail!("DNS 提供商尚未配置。请在「系统设置」-「ACME配置」中配置DNS提供商信息。")
    }

    let (cert_pem, key_pem, expires_at) = issue_cert(&req.domain, &acme_config).await?;

    let cert_id = id::next();
    let now = tools::now();

    let cert = TlsCertBuilder::default()
        .id(Some(cert_id))
        .domain(Some(req.domain.clone()))
        .cert_pem(Some(cert_pem))
        .key_pem(Some(key_pem))
        .issuer(Some("Let's Encrypt".to_string()))
        .issued_at(Some(now.clone()))
        .expires_at(Some(rbatis::rbdc::DateTime::from_timestamp(expires_at)))
        .auto_renew(Some(req.auto_renew))
        .create_user_id(Some(user.id))
        .create_time(Some(now))
        .remark(req.remark)
        .build()?;

    TlsCert::insert(Pool::get()?, &cert).await?;
    log::info!("Certificate issued for domain: {}", req.domain);

    Ok(cert_id)
}

/// 续期证书
pub async fn renew(req: IdReq, _user: UserPrincipal) -> anyhow::Result<i64> {
    let rows = TlsCert::select_by_map(Pool::get()?, value! { "id": req.id }).await?;
    let old = rows.into_iter().next().context("Certificate not found")?;
    let domain = old.domain.context("Domain is empty")?;

    let acme_config = acme_service::get_raw().await?;
    let (cert_pem, key_pem, expires_at) = issue_cert(&domain, &acme_config).await?;

    update_cert_record(req.id, &cert_pem, &key_pem, expires_at).await?;
    update_domain_cert(&domain, &cert_pem, &key_pem).await?;

    log::info!("Certificate renewed for domain: {}", domain);
    Ok(req.id)
}

/// 证书列表
pub async fn list(req: CertListReq) -> anyhow::Result<PageRes<CertListRes>> {
    let page =
        crate::server::db::models::tls_cert::list_page(Pool::get()?, &req.to_rb_page(), &req)
            .await?;
    let list =
        page.convert_to_page_res(|records| records.into_iter().map(CertListRes::from).collect());
    Ok(list)
}

/// 证书详情
pub async fn detail(id: i64) -> anyhow::Result<CertDetailRes> {
    let rows = TlsCert::select_by_map(Pool::get()?, value! { "id": id }).await?;
    let cert = rows.into_iter().next().context("Certificate not found")?;
    Ok(CertDetailRes::from(cert))
}

/// 删除证书
pub async fn delete(req: IdsReq) -> anyhow::Result<()> {
    TlsCert::delete_by_map(Pool::get()?, value! { "id": req.ids }).await?;
    Ok(())
}

/// 获取证书和私钥
pub async fn get_cert_key(id: i64) -> anyhow::Result<CertKeyRes> {
    let rows = TlsCert::select_by_map(Pool::get()?, value! { "id": id }).await?;
    let cert = rows.into_iter().next().context("Certificate not found")?;
    Ok(CertKeyRes::from(cert))
}

/// 设置自动续期
pub async fn set_auto_renew(req: SetAutoRenewReq) -> anyhow::Result<()> {
    TlsCert::update_by_map(
        Pool::get()?,
        &TlsCertBuilder::default()
            .auto_renew(Some(req.auto_renew))
            .update_time(Some(tools::now()))
            .build()?,
        value! { "id": req.id },
    )
    .await?;
    Ok(())
}

/// 自动续期
pub async fn auto_renew() -> anyhow::Result<()> {
    let acme_config = acme_service::get_raw().await?;

    let certs = TlsCert::select_by_map(
        Pool::get()?,
        value! {
            "auto_renew": 1,
        },
    )
    .await?;

    let now_ms = chrono::Utc::now().timestamp_millis();
    let threshold_ms = now_ms + 7 * 24 * 60 * 60 * 1000;

    for cert in certs {
        let expires_at = match &cert.expires_at {
            Some(dt) => dt.unix_timestamp() * 1000,
            None => continue,
        };
        if expires_at > threshold_ms {
            continue;
        }

        let domain = match &cert.domain {
            Some(d) => d.clone(),
            None => continue,
        };

        log::info!("Auto renewing cert for domain: {}", domain);

        if let Err(e) = renew_one(&cert, &acme_config).await {
            log::error!("Failed to auto renew cert for domain {}: {}", domain, e);
            continue;
        }
    }

    Ok(())
}

/// 续期单个证书
async fn renew_one(old_cert: &TlsCert, acme_config: &AcmeConfig) -> anyhow::Result<i64> {
    let domain = match &old_cert.domain {
        Some(d) => d.clone(),
        None => bail!("Certificate domain not found"),
    };
    let cert_id = match &old_cert.id {
        Some(id) => *id,
        None => bail!("Certificate id not found"),
    };
    let (cert_pem, key_pem, expires_at) = issue_cert(&domain, acme_config).await?;

    update_cert_record(cert_id, &cert_pem, &key_pem, expires_at).await?;
    update_domain_cert(&domain, &cert_pem, &key_pem).await?;

    log::info!("Certificate renewed for domain: {}", domain);
    Ok(cert_id)
}

/// 调用 ACME 签发证书
async fn issue_cert(
    domain: &str,
    acme_config: &AcmeConfig,
) -> anyhow::Result<(String, String, i64)> {
    let dns_provider = build_dns_provider(acme_config)?;

    let account_key = acme_service::ensure_account(acme_config).await?;
    let issuer =
        CertIssuer::from_key(&acme_config.email, acme_config.staging, &account_key).await?;
    let result = issuer
        .issue(&[domain], &dns_provider, 15)
        .await
        .map_err(|e| anyhow::anyhow!("Certificate issuance failed: {}", e))?;

    Ok((result.cert_pem, result.key_pem, result.expires_at))
}

/// 更新证书记录
async fn update_cert_record(
    id: i64,
    cert_pem: &str,
    key_pem: &str,
    expires_at: i64,
) -> anyhow::Result<()> {
    let now = tools::now();
    TlsCert::update_by_map(
        Pool::get()?,
        &TlsCertBuilder::default()
            .cert_pem(Some(cert_pem.to_string()))
            .key_pem(Some(key_pem.to_string()))
            .expires_at(Some(rbatis::rbdc::DateTime::from_timestamp(expires_at)))
            .issued_at(Some(now.clone()))
            .update_time(Some(now))
            .build()?,
        value! { "id": id },
    )
    .await?;
    Ok(())
}

/// 更新 domain 表关联的证书字段
async fn update_domain_cert(domain: &str, cert_pem: &str, key_pem: &str) -> anyhow::Result<()> {
    let domains = Domain::select_by_map(
        Pool::get()?,
        value! {
            "domain": domain
        },
    )
    .await?;
    for d in domains {
        Domain::update_by_map(
            Pool::get()?,
            &DomainBuilder::default()
                .cert(Some(cert_pem.to_string()))
                .cert_key(Some(key_pem.to_string()))
                .update_time(Some(tools::now()))
                .build()?,
            value! { "id": d.id },
        )
        .await?;
    }
    Ok(())
}

/// 根据系统配置中的 DNS 提供商及凭证构建 DnsProvider
fn build_dns_provider(config: &AcmeConfig) -> anyhow::Result<DnsProvider> {
    match (&config.dns_provider[..], &config.dns_credentials) {
        ("Cloudflare", DnsCredentials::Cloudflare { api_token }) => Ok(DnsProvider::Cloudflare {
            api_token: api_token.clone(),
        }),
        (
            "Tencent",
            DnsCredentials::Tencent {
                secret_id,
                secret_key,
            },
        ) => Ok(DnsProvider::Tencent {
            secret_id: secret_id.clone(),
            secret_key: secret_key.clone(),
        }),
        (
            "Ali",
            DnsCredentials::Ali {
                access_key,
                secret_key,
            },
        ) => Ok(DnsProvider::Ali {
            access_key: access_key.clone(),
            secret_key: secret_key.clone(),
        }),
        (
            "Huawei",
            DnsCredentials::Huawei {
                access_key,
                secret_key,
            },
        ) => Ok(DnsProvider::Huawei {
            access_key: access_key.clone(),
            secret_key: secret_key.clone(),
        }),
        (
            "Volcengine",
            DnsCredentials::Volcengine {
                access_key,
                secret_key,
            },
        ) => Ok(DnsProvider::Volcengine {
            access_key: access_key.clone(),
            secret_key: secret_key.clone(),
        }),
        (
            "Baidu",
            DnsCredentials::Baidu {
                access_key,
                secret_key,
            },
        ) => Ok(DnsProvider::Baidu {
            access_key: access_key.clone(),
            secret_key: secret_key.clone(),
        }),
        (provider, _) => bail!("DNS 提供商 '{}' 未配置或凭证类型不匹配", provider),
    }
}
