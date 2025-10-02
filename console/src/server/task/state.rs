use crate::server::db::models::gateway_node::{GatewayNode, GatewayNodeBuilder, GatewayNodeStatus};
use crate::server::db::{Pool, tools};
use logging::log;
use rbs::value;
use std::time::Duration;

/// 更新心跳超时的节点状态为Offline
///
/// 超时时间：2 × 节点上报间隔时间
pub async fn update_timeout_heartbeat_node() {
    let tx = Pool::get().unwrap();
    let nodes = GatewayNode::select_by_map(
        tx,
        value! {
            "status":GatewayNodeStatus::Online
        },
    )
    .await
    .unwrap();
    if nodes.is_empty() {
        return;
    }
    for node in nodes {
        if tools::now().sub(Duration::from_secs(
            common::constants::REPORT_STATE_INTERVAL * 2,
        )) > node.last_heartbeat_time.unwrap()
        {
            log::warn!(
                "node {} heartbeat timeout, update status to offline",
                node.node_id.clone().unwrap()
            );
            let update = GatewayNodeBuilder::default()
                .id(node.id)
                .status(Some(GatewayNodeStatus::Offline))
                .status_msg(Some("heartbeat timeout".to_string()))
                .update_time(Some(tools::now()))
                .build()
                .unwrap();
            if let Err(e) = GatewayNode::update_by_map(tx, &update, value! {"id":node.id}).await {
                log::error!("update_heartbeat error:{}", e);
            }
        }
    }
}
