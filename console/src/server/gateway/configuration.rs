use crate::server::db::Pool;
use crate::server::db::models::plugin::Plugin;
use protocol::gateway;

pub(crate) async fn configuration() -> anyhow::Result<gateway::Configuration> {
    //TODO
    Ok(gateway::Configuration {
        pre_filters: vec![],
        post_filters: vec![],
    })
}
