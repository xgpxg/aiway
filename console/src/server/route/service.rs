use crate::server::auth::UserPrincipal;
use crate::server::db::models::route;
use crate::server::db::models::route::{Route, RouteBuilder, RouteStatus};
use crate::server::db::models::system_config::{ConfigKey, SystemConfig};
use crate::server::db::{Pool, tools};
use crate::server::route::PathPatterns;
use crate::server::route::request::{
    RouteAddOrUpdateReq, RouteListReq, UpdateGlobalFilterConfigReq, UpdateStatusReq,
};
use crate::server::route::response::RouteListRes;
use anyhow::{Context, bail};
use common::id;
use busi::req::{IdsReq, Pagination};
use busi::res::{IntoPageRes, PageRes};
use aiway_protocol::gateway::GlobalFilter;
use rbs::value;

pub async fn add(req: RouteAddOrUpdateReq, user: UserPrincipal) -> anyhow::Result<()> {
    let route = Route {
        id: Some(id::next()),
        status: Some(RouteStatus::Disable),
        create_user_id: Some(user.id),
        create_time: Some(tools::now()),
        ..Route::from(req)
    };

    check_exists(&route, None).await?;

    Route::insert(Pool::get()?, &route).await?;
    Ok(())
}

/// 检查路由是否存在
///
/// 唯一路由：Host + Path 唯一，且同一 Host 下的 Path 不能冲突。
/// 由于 Host 不是必须的，所以在检查是否冲突时需要处理 None 的情况
async fn check_exists(route: &Route, exclude_id: Option<i64>) -> anyhow::Result<()> {
    let host = route.host.as_deref();
    let path = route.path.as_deref();
    let tx = Pool::get()?;
    let mut list: Vec<Route> = if host.is_some() {
        tx.query_decode("select * from route where host = ?", vec![value!(host)])
            .await?
    } else {
        tx.query_decode("select * from route where host is null", vec![])
            .await?
    };

    // 移除排除的 ID，一般用来忽略自身
    list.retain(|item| item.id != exclude_id);

    let mut paths = vec![path.map(|s| s.to_string()).context("Route path required")?];
    paths.extend(
        list.into_iter()
            .map(|item| item.path.unwrap())
            .collect::<Vec<_>>(),
    );
    if let Err(e) = matchit::Router::try_from(PathPatterns::new(paths)) {
        bail!("路径验证失败：{}", e);
    }

    Ok(())
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
    let old = old.first().unwrap();
    let id = req.id.context("ID cannot be empty")?;
    let new = Route::from(req);
    let update = Route {
        update_user_id: Some(user.id),
        update_time: Some(tools::now()),
        status: old.status.clone(),
        ..new
    };

    check_exists(&update, Some(id)).await?;

    Route::update_by_map(Pool::get()?, &update, value! { "id":id}).await?;
    Ok(())
}

pub async fn delete(req: IdsReq, _user: UserPrincipal) -> anyhow::Result<()> {
    Route::delete_by_map(Pool::get()?, value! { "id": req.ids }).await?;
    Ok(())
}

pub async fn update_status(req: UpdateStatusReq, user: UserPrincipal) -> anyhow::Result<()> {
    let old = Route::select_by_map(Pool::get()?, value! { "id": req.id}).await?;
    if old.is_empty() {
        bail!("Route not found")
    }
    Route::update_by_map(
        Pool::get()?,
        &RouteBuilder::default()
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

pub async fn update_global_filter_config(
    req: UpdateGlobalFilterConfigReq,
    _user: UserPrincipal,
) -> anyhow::Result<()> {
    SystemConfig::upsert(ConfigKey::GlobalFilter, &req.inner).await?;
    Ok(())
}

pub(crate) async fn get_global_filter_config(_user: UserPrincipal) -> anyhow::Result<GlobalFilter> {
    SystemConfig::get(ConfigKey::GlobalFilter).await
}
