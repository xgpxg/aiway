use crate::server::auth::UserPrincipal;
use crate::server::db::models::route::{Route, RouteBuilder};
use crate::server::db::{Pool, tools};
use crate::server::route::request::RouteAddReq;
use anyhow::bail;
use common::id;
use rbs::value;

pub async fn add(req: RouteAddReq, user: UserPrincipal) -> anyhow::Result<()> {
    let route = RouteBuilder::default()
        .id(Some(id::next()))
        .name(Some(req.name))
        .description(req.description)
        .host(req.host)
        .prefix(req.prefix)
        .path(req.path.into())
        .service(req.service.into())
        .header(req.header.into())
        .query(req.query.into())
        .pre_filters(req.pre_filters.into())
        .post_filters(req.post_filters.into())
        .create_user_id(Some(user.id))
        .create_time(Some(tools::now()))
        .build()?;
    if check_exists(&route).await? {
        bail!("路由已存在")
    }
    Route::insert(Pool::get()?, &route).await?;
    Ok(())
}

async fn check_exists(route: &Route) -> anyhow::Result<bool> {
    let list = Route::select_by_map(
        Pool::get()?,
        value! {
            "host": &route.host,
            "prefix": &route.prefix,
            "path": &route.path,
        },
    )
    .await?;
    Ok(!list.is_empty())
}
