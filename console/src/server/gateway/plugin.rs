use crate::server::db::Pool;
use crate::server::db::models::plugin::Plugin;

pub(crate) async fn plugins() -> anyhow::Result<Vec<protocol::gateway::Plugin>> {
    let plugins = Plugin::select_all(Pool::get()?).await?;
    let mut list = Vec::with_capacity(plugins.len());
    for plugin in plugins {
        list.push(protocol::gateway::Plugin {
            name: plugin.name.unwrap(),
            phase: plugin.phase.unwrap(),
            url: plugin.url.unwrap(),
            version: plugin.version.unwrap(),
        });
    }
    Ok(list)
}
