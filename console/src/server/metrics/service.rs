use crate::server::db::Pool;
use crate::server::db::models::gateway_node::GatewayNode;
use crate::server::db::models::gateway_node_state::GatewayNodeState;
use crate::server::metrics::response::GatewayState;

pub async fn gateway_state() -> anyhow::Result<GatewayState> {
    let mut state = GatewayState::default();

    let tx = Pool::get()?;

    // 查询所有节点
    let nodes = GatewayNode::select_all(tx).await?;
    // 网关节点数
    state.node_count = nodes.len();
    state.online_node_count = nodes.iter().filter(|n| n.is_online()).count();
    state.offline_node_count = nodes.iter().filter(|n| n.is_offline()).count();

    // 节点ID
    let node_ids = nodes
        .iter()
        .map(|n| n.node_id.clone().unwrap())
        .collect::<Vec<_>>();
    // 在线的节点ID
    let online_node_ids = nodes
        .iter()
        .filter(|n| n.is_online())
        .map(|n| n.node_id.clone().unwrap())
        .collect::<Vec<_>>();
    // 每个节点最新状态
    let node_states: Vec<GatewayNodeState> = tx
        .query_decode(
            &format!(
                r#"
            select gns.* from gateway_node_state gns inner join (
                select max(id) as id from gateway_node_state where node_id in ({}) group by node_id
            ) as t on t.id = gns.id
            "#,
                &node_ids
                    .iter()
                    .map(|id| format!("'{}'", id))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            vec![],
        )
        .await?;
    // 在线节点最新状态
    let online_node_states = node_states
        .iter()
        .filter(|s| online_node_ids.contains(&s.node_id))
        .collect::<Vec<_>>();

    // 网关整体的平均QPS
    state.avg_qps = online_node_states.iter().map(|s| s.avg_qps).sum::<usize>();

    // 网关整体的平均响应时间
    if online_node_ids.len() > 0 {
        state.avg_response_time = online_node_states
            .iter()
            .map(|s| s.avg_response_time)
            .sum::<usize>()
            / online_node_ids.len();
    }

    state.request_count = node_states.iter().map(|s| s.request_count).sum::<usize>();
    state.request_invalid_count = node_states
        .iter()
        .map(|s| s.request_invalid_count)
        .sum::<usize>();
    state.response_2xx_count = node_states
        .iter()
        .map(|s| s.response_2xx_count)
        .sum::<usize>();
    state.response_3xx_count = node_states
        .iter()
        .map(|s| s.response_3xx_count)
        .sum::<usize>();
    state.response_4xx_count = node_states
        .iter()
        .map(|s| s.response_4xx_count)
        .sum::<usize>();
    state.response_5xx_count = node_states
        .iter()
        .map(|s| s.response_5xx_count)
        .sum::<usize>();

    Ok(state)
}
