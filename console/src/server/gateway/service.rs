use crate::server::db::Pool;
use crate::server::db::models::service::Service;

pub(crate) async fn services() -> anyhow::Result<Vec<protocol::gateway::Service>> {
    let services = Service::select_all(Pool::get()?).await?;
    let mut list = Vec::with_capacity(services.len());
    for service in services {
        list.push(protocol::gateway::Service {
            name: service.name.unwrap(),
            nodes: service.nodes.unwrap(),
            lb: service.lb.unwrap(),
        });
    }
    Ok(list)
}
