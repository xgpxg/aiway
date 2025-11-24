use crate::server::db::Pool;
use crate::server::db::models::gateway_node::GatewayNode;
use crate::server::db::models::gateway_node_state::GatewayNodeState;
use crate::server::metrics::response::GatewayState;
use chrono::Timelike;
use rbs::value;
use serde::Deserialize;

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
    // let node_ids = nodes
    //     .iter()
    //     .map(|n| n.node_id.clone().unwrap())
    //     .collect::<Vec<_>>();
    // 在线的节点ID
    let online_node_ids = nodes
        .iter()
        .filter(|n| n.is_online())
        .map(|n| n.node_id.clone().unwrap())
        .collect::<Vec<_>>();
    // 每个节点最新状态
    let node_states: Vec<GatewayNodeState> = tx
        .query_decode(
            r#"
            select gns.* from gateway_node_state gns inner join (
                select max(id) as id from gateway_node_state group by node_id
            ) as t on t.id = gns.id
            "#,
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
    if !online_node_ids.is_empty() {
        state.avg_response_time = online_node_states
            .iter()
            .map(|s| s.avg_response_time)
            .sum::<usize>()
            / online_node_ids.len();
    }

    // 今日请求数
    let start_of_day = chrono::Local::now()
        .with_hour(0)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap()
        .with_nanosecond(0)
        .unwrap()
        .timestamp_millis();

    let request_today_count = tx
        .query_decode::<Option<u64>>(
            r#"
           select sum(interval_request_count) from gateway_node_state where ts >= ? group by node_id
            "#,
            vec![value!(start_of_day)],
        )
        .await?
        .unwrap_or(0) as usize;

    state.request_today_count = request_today_count;

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
    state.http_connect_count = node_states
        .iter()
        .map(|s| s.http_connect_count as usize)
        .sum::<usize>();
    state.net_rx = node_states.iter().map(|s| s.net_rx as usize).sum::<usize>();
    state.net_tx = node_states.iter().map(|s| s.net_tx as usize).sum::<usize>();

    // info、warn、error级别的未读消息数
    #[derive(Deserialize)]
    struct MessageCount {
        info_count: Option<usize>,
        warn_count: Option<usize>,
        error_count: Option<usize>,
    }
    let message_count = tx
        .query_decode::<MessageCount>(
            r#"
                SELECT
                    SUM(CASE WHEN level = 'Info' THEN 1 ELSE 0 END) as info_count,
                    SUM(CASE WHEN level = 'Warn' THEN 1 ELSE 0 END) as warn_count,
                    SUM(CASE WHEN level = 'Error' THEN 1 ELSE 0 END) as error_count
                FROM message
                WHERE read_status = 'Unread'
            "#,
            vec![],
        )
        .await?;

    state.info_count = message_count.info_count.unwrap_or(0);
    state.warn_count = message_count.warn_count.unwrap_or(0);
    state.error_count = message_count.error_count.unwrap_or(0);

    // 最新消息标题
    state.last_message_title = tx
        .query_decode::<Option<String>>(
            r#"
                SELECT title FROM message ORDER BY id DESC LIMIT 1
            "#,
            vec![],
        )
        .await?;

    Ok(state)
}
