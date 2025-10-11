use crate::server::auth::UserPrincipal;
use crate::server::db::models::service;
use crate::server::db::models::service::{Service, ServiceBuilder, ServiceStatus};
use crate::server::db::{Pool, tools};
use crate::server::service::request::{ServiceAddOrUpdateReq, ServiceListReq, UpdateStatusReq};
use crate::server::service::response::ServiceListRes;
use anyhow::{Context, bail};
use common::id;
use protocol::common::req::{IdsReq, Pagination};
use protocol::common::res::{IntoPageRes, PageRes};
use rbs::value;

pub async fn add(req: ServiceAddOrUpdateReq, user: UserPrincipal) -> anyhow::Result<()> {
    let service = Service {
        id: Some(id::next()),
        status: Some(ServiceStatus::Disable),
        create_user_id: Some(user.id),
        create_time: Some(tools::now()),
        ..Service::from(req)
    };
    if check_exists(&service, None).await? {
        bail!("Service with name {} already exists", service.name.unwrap())
    }
    Service::insert(Pool::get()?, &service).await?;
    Ok(())
}
async fn check_exists(service: &Service, exclude_id: Option<i64>) -> anyhow::Result<bool> {
    let mut list = Service::select_by_map(
        Pool::get()?,
        value! {
            "name": &service.name,
        },
    )
    .await?;

    list.retain(|item| item.id != exclude_id);

    Ok(!list.is_empty())
}

pub async fn list(req: ServiceListReq) -> anyhow::Result<PageRes<ServiceListRes>> {
    let page = service::list_page(Pool::get()?, &req.to_rb_page(), &req).await?;
    let list = page.convert_to_page_res(|list| {
        list.into_iter()
            .map(|item| ServiceListRes { inner: item })
            .collect::<Vec<_>>()
    });
    Ok(list)
}

pub async fn update(req: ServiceAddOrUpdateReq, user: UserPrincipal) -> anyhow::Result<()> {
    let old = Service::select_by_map(Pool::get()?, value! { "id": req.id}).await?;
    if old.is_empty() {
        bail!("Service not found");
    }
    let id = req.id.context("ID cannot be empty")?;
    let new = Service::from(req);
    let update = Service {
        update_user_id: Some(user.id),
        update_time: Some(tools::now()),
        ..new
    };

    if check_exists(&update, Some(id)).await? {
        bail!("Service already exists")
    }

    Service::update_by_map(Pool::get()?, &update, value! { "id":id}).await?;
    Ok(())
}

pub async fn delete(req: IdsReq) -> anyhow::Result<()> {
    Service::delete_by_map(Pool::get()?, value! { "id": req.ids}).await?;
    Ok(())
}

pub(crate) async fn update_status(req: UpdateStatusReq, user: UserPrincipal) -> anyhow::Result<()> {
    let old = Service::select_by_map(Pool::get()?, value! { "id": req.id}).await?;
    if old.is_empty() {
        bail!("Service not found")
    }
    Service::update_by_map(
        Pool::get()?,
        &ServiceBuilder::default()
            .id(Some(req.id))
            .status(Some(req.status))
            .update_user_id(Some(user.id))
            .update_time(Some(tools::now()))
            .build()?,
        value! { "id": req.id},
    )
    .await?;
    Ok(())
}
