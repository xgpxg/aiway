use crate::server::db::Pool;
use crate::server::db::models::route::{Route, RouteStatus};
use crate::server::route::PathPattern;
use rbs::value;

pub(crate) async fn routes() -> anyhow::Result<Vec<protocol::gateway::Route>> {
    let routes = Route::select_by_map(Pool::get()?, value! {"status": RouteStatus::Ok}).await?;
    let mut list = Vec::with_capacity(routes.len());
    for route in routes {
        list.push(protocol::gateway::route::Route {
            name: route.name.unwrap(),
            host: route.host.unwrap(),
            path: route.path.clone().unwrap(),
            // 由路径重写插件实现
            //strip_prefix: route.strip_prefix.unwrap() == 1,
            match_path: PathPattern::new(route.path.unwrap()).to_pattern(),
            service: route.service.unwrap(),
            methods: route.methods.unwrap_or_default(),
            header: route.header.unwrap_or_default(),
            query: route.query.unwrap_or_default(),
            pre_filters: route.pre_filters.unwrap_or_default(),
            post_filters: route.post_filters.unwrap_or_default(),
            is_auth: route.is_auth.unwrap_or_default(),
            auth_white_list: route.auth_white_list.unwrap_or_default(),
        });
    }

    // pre_filters和post_filters中对应的插件配置已经由前端传入，不需要再组装插件的默认配置。
    // 插件的配置应该在控制台构建，而不应该在网关构建，网关仅负责执行。

    Ok(list)
}

#[cfg(test)]
mod tests {
    //use config::{Config, FileFormat};
    #[test]
    fn test_config() {
        /* let config = Config::builder()
            .add_source(config::File::from_str(
                "name: aaa\naddress: \n  city: 123",
                FileFormat::Yaml,
            ))
            .add_source(config::File::from_str(
                "name: bbb\naddress: \n  city: 123",
                FileFormat::Yaml,
            ))
            .build()
            .unwrap();
        let config = config.try_deserialize::<serde_json::Value>().unwrap();
        println!("{:?}", config);
        println!("{:?}", config);*/
    }
}
