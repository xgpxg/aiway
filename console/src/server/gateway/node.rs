use crate::server::db::Pool;
use crate::server::db::models::gateway_node::{GatewayNode, GatewayNodeStatus};
use aiway_protocol::gateway::GatewayNodeInfo;
use rbs::value;

/// 查询所有在线的网关节点，供接入层发现可用节点
pub(crate) async fn online_nodes() -> anyhow::Result<Vec<GatewayNodeInfo>> {
    let nodes = GatewayNode::select_by_map(
        Pool::get()?,
        value! {"status": GatewayNodeStatus::Online},
    )
    .await?;

    let mut list = Vec::with_capacity(nodes.len());
    for node in nodes {
        let ip = match node.ip {
            Some(ip) if !ip.is_empty() => ip,
            _ => continue,
        };
        let port = match node.port {
            Some(port) if port > 0 => port,
            _ => continue,
        };
        list.push(GatewayNodeInfo { ip, port });
    }
    Ok(list)
}
