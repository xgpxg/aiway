use crate::server::auth::UserPrincipal;
use crate::server::db::models::domain::{Domain, DomainBuilder, DomainStatus, Protocol};
use crate::server::db::{tools, Pool};
use crate::server::domain::request::{DomainAddOrUpdateReq, UpdateStatusReq};
use crate::server::domain::response::DomainListRes;
use crate::server::domain::DomainListReq;
use anyhow::{Context, bail};
use busi::req::{IdsReq, Pagination};
use busi::res::{IntoPageRes, PageRes};
use common::id;
use rbs::value;

pub async fn add(req: DomainAddOrUpdateReq, user: UserPrincipal) -> anyhow::Result<()> {
    let protocol = parse_protocol(&req.protocol)?;
    let (cert, cert_key) = validate_cert(&protocol, &req.cert, &req.key)?;

    let domain = Domain {
        id: Some(id::next()),
        domain: Some(req.domain),
        protocol: Some(protocol),
        cert,
        cert_key,
        status: Some(DomainStatus::Ok),
        create_user_id: Some(user.id),
        create_time: Some(tools::now()),
        remark: req.remark,
        ..Domain::default()
    };

    Domain::insert(Pool::get()?, &domain).await?;
    Ok(())
}

pub async fn update(req: DomainAddOrUpdateReq, user: UserPrincipal) -> anyhow::Result<()> {
    let domain_id = req.id.context("ID cannot be empty")?;
    let old = Domain::select_by_map(Pool::get()?, value! { "id": domain_id }).await?;
    if old.is_empty() {
        bail!("Domain not found")
    }

    let protocol = parse_protocol(&req.protocol)?;
    let (cert, cert_key) = validate_cert(&protocol, &req.cert, &req.key)?;

    let update = Domain {
        domain: Some(req.domain),
        protocol: Some(protocol),
        cert,
        cert_key,
        update_user_id: Some(user.id),
        update_time: Some(tools::now()),
        remark: req.remark,

        ..Domain::default()
    };

    Domain::update_by_map(Pool::get()?, &update, value! { "id": domain_id }).await?;
    Ok(())
}

pub async fn delete(req: IdsReq) -> anyhow::Result<()> {
    Domain::delete_by_map(Pool::get()?, value! { "id": req.ids }).await?;
    Ok(())
}

pub async fn update_status(req: UpdateStatusReq, user: UserPrincipal) -> anyhow::Result<()> {
    let old = Domain::select_by_map(Pool::get()?, value! { "id": req.id }).await?;
    if old.is_empty() {
        bail!("Domain not found")
    }

    Domain::update_by_map(
        Pool::get()?,
        &DomainBuilder::default()
            .id(Some(req.id))
            .status(Some(req.status))
            .update_user_id(Some(user.id))
            .update_time(Some(tools::now()))
            .build()?,
        value! { "id": req.id },
    )
    .await?;
    Ok(())
}

pub async fn list(req: DomainListReq) -> anyhow::Result<PageRes<DomainListRes>> {
    let page =
        crate::server::db::models::domain::list_page(Pool::get()?, &req.to_rb_page(), &req).await?;
    let list = page.convert_to_page_res(|list| {
        list.into_iter()
            .map(|item| DomainListRes { inner: item })
            .collect::<Vec<_>>()
    });
    Ok(list)
}

fn parse_protocol(s: &str) -> anyhow::Result<Protocol> {
    match s.to_uppercase().as_str() {
        "HTTP" => Ok(Protocol::HTTP),
        "HTTPS" => Ok(Protocol::HTTPS),
        _ => bail!("Invalid protocol: {}, expected HTTP or HTTPS", s),
    }
}

fn validate_cert(
    protocol: &Protocol,
    cert: &Option<String>,
    key: &Option<String>,
) -> anyhow::Result<(Option<String>, Option<String>)> {
    if *protocol == Protocol::HTTPS {
        let cert = cert
            .as_ref()
            .filter(|s| !s.is_empty())
            .context("Certificate is required for HTTPS")?
            .clone();
        let key = key
            .as_ref()
            .filter(|s| !s.is_empty())
            .context("Private key is required for HTTPS")?
            .clone();
        Ok((Some(cert), Some(key)))
    } else {
        Ok((None, None))
    }
}
