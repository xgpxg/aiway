use crate::server::db::models::route::Route;
use crate::server::db::Pool;

pub(crate) async fn routes() -> anyhow::Result<Vec<protocol::gateway::Route>> {
    let routes = Route::select_all(Pool::get()?).await?;
    let mut list = Vec::with_capacity(routes.len());
    for route in routes {
        list.push(protocol::gateway::route::Route {
            name: route.name.unwrap(),
            prefix: route.prefix,
            path: route.path.unwrap(),
            strip_prefix: route.strip_prefix.unwrap() == 1,
            service: route.service.unwrap(),
            header: route.header.unwrap_or_default(),
            query: route.query.unwrap_or_default(),
            pre_filters: route.pre_filters.unwrap_or_default(),
            post_filters: route.post_filters.unwrap_or_default(),
        });
    }
    Ok(list)
}
