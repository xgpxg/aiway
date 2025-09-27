use crate::server::auth::UserPrincipal;
use crate::server::db::models::route;
use crate::server::db::models::route::{Route, RouteStatus};
use crate::server::db::{Pool, tools};
use crate::server::route::request::{RouteAddOrUpdateReq, RouteListReq};
use crate::server::route::response::RouteListRes;
use anyhow::{Context, bail};
use common::id;
use protocol::common::req::{IdsReq, Pagination};
use protocol::common::res::{IntoPageRes, PageRes};
use rbs::value;

pub async fn add(req: RouteAddOrUpdateReq, user: UserPrincipal) -> anyhow::Result<()> {
    let route = Route {
        id: Some(id::next()),
        status: Some(RouteStatus::Disable),
        create_user_id: Some(user.id),
        create_time: Some(tools::now()),
        ..Route::from(req)
    };
    if check_exists(&route, None).await? {
        bail!("Route already exists")
    }

    Route::insert(Pool::get()?, &route).await?;
    Ok(())
}

async fn check_exists(route: &Route, exclude_id: Option<i64>) -> anyhow::Result<bool> {
    let mut list = Route::select_by_map(
        Pool::get()?,
        value! {
            "host": &route.host,
            "prefix": &route.prefix,
            "path": &route.path,
        },
    )
    .await?;

    list.retain(|item| item.id != exclude_id);

    Ok(!list.is_empty())
}

pub async fn list(
    req: RouteListReq,
    _user: UserPrincipal,
) -> anyhow::Result<PageRes<RouteListRes>> {
    let page = route::list_page(Pool::get()?, &req.to_rb_page(), &req).await?;
    let list = page.convert_to_page_res(|list| {
        list.into_iter()
            .map(|item| RouteListRes { inner: item })
            .collect::<Vec<_>>()
    });
    Ok(list)
}

pub async fn update(req: RouteAddOrUpdateReq, user: UserPrincipal) -> anyhow::Result<()> {
    let old = Route::select_by_map(Pool::get()?, value! { "id": req.id}).await?;
    if old.is_empty() {
        bail!("Route not found")
    }
    let id = req.id.context("ID cannot be empty")?;
    let new = Route::from(req);
    let update = Route {
        update_user_id: Some(user.id),
        update_time: Some(tools::now()),
        ..new
    };

    if check_exists(&update, Some(id)).await? {
        bail!("Route already exists")
    }

    Route::update_by_map(Pool::get()?, &update, value! { "id":id}).await?;
    Ok(())
}

pub async fn delete(req: IdsReq, _user: UserPrincipal) -> anyhow::Result<()> {
    Route::delete_by_map(Pool::get()?, value! { "id": req.ids }).await?;
    Ok(())
}
