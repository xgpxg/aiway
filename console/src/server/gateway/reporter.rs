use crate::server::db::models::gateway_node::{GatewayNode, GatewayNodeBuilder, GatewayNodeStatus};
use crate::server::db::models::gateway_node_state::{GatewayNodeState, GatewayNodeStateBuilder};
use crate::server::db::{Pool, tools};
use alert::Alert;
use common::id;
use logging::log;
use aiway_protocol::gateway::state::State;
use rbs::value;

/// 接收gateway上报数据
///
/// 注意这里仅接收网关节点的状态数据，告警数据使用alert接口处理。
pub async fn report(req: State) -> anyhow::Result<()> {
    let node_id = &req.node_info.node_id;

    log::debug!("node_id:{}, state: {:?}", node_id, req);

    let tx = Pool::get()?;
    let gateway_node = GatewayNode::select_by_map(tx, value! {"node_id": node_id}).await?;
    if gateway_node.is_empty() {
        // 新增
        let gateway_node = GatewayNodeBuilder::default()
            .id(Some(id::next()))
            .node_id(Some(node_id.clone()))
            .ip(Some(req.node_info.ip))
            .port(Some(req.node_info.port))
            .status(Some(GatewayNodeStatus::Online))
            .last_heartbeat_time(Some(tools::now()))
            .build()?;
        GatewayNode::insert(tx, &gateway_node).await?;
        // 发送提醒
        send_alert(&gateway_node);
    } else {
        // 更新节点心跳时间
        let gateway_node = gateway_node.first().unwrap().clone();
        // 发送提醒
        if gateway_node.status.as_ref().unwrap() == &GatewayNodeStatus::Offline {
            send_alert(&gateway_node);
        }
        let gateway_node = GatewayNodeBuilder::default()
            .last_heartbeat_time(Some(tools::now()))
            .status(Some(GatewayNodeStatus::Online))
            .update_time(Some(tools::now()))
            .build()?;
        GatewayNode::update_by_map(tx, &gateway_node, value! {"node_id": node_id}).await?;
    }

    //上一条state
    let last: Option<GatewayNodeState> = tx
        .query_decode(
            "select * from gateway_node_state where node_id = ? order by id desc limit 1",
            vec![value!(node_id)],
        )
        .await?;

    let last = last.unwrap_or_default();

    let gateway_state_log = GatewayNodeStateBuilder::default()
        .id(Some(id::next()))
        .node_id(node_id.clone())
        .ts(req.timestamp)
        .os(Some(req.system_state.os))
        .cpu_usage(req.system_state.cpu_usage)
        .mem_total(req.system_state.mem_state.total)
        .mem_free(req.system_state.mem_state.free)
        .mem_used(req.system_state.mem_state.used)
        .disk_total(req.system_state.disk_state.total)
        .disk_free(req.system_state.disk_state.free)
        .net_rx(req.system_state.net_state.rx)
        .net_tx(req.system_state.net_state.tx)
        .net_tcp_conn_count(req.system_state.net_state.tcp_conn_count)
        .http_connect_count(req.moment_counter.http_connect_count)
        .sse_connect_count(req.moment_counter.sse_connect_count)
        .avg_qps(if req.counter.response_time_since_last > 0 {
            req.counter.request_count / common::constants::REPORT_STATE_INTERVAL as usize
        } else {
            0
        })
        // 上报区间内的统计
        .interval_request_count(req.counter.request_count)
        .interval_request_invalid_count(req.counter.request_invalid_count)
        .interval_response_2xx_count(req.counter.response_2xx_count)
        .interval_response_3xx_count(req.counter.response_3xx_count)
        .interval_response_4xx_count(req.counter.response_4xx_count)
        .interval_response_5xx_count(req.counter.response_5xx_count)
        .interval_avg_response_time(if req.counter.request_count > 0 {
            req.counter.response_time_since_last / req.counter.request_count
        } else {
            0
        })
        // 累计统计
        .request_count(req.counter.request_count + last.request_count)
        .request_invalid_count(req.counter.request_invalid_count + last.request_invalid_count)
        .response_2xx_count(req.counter.response_2xx_count + last.response_2xx_count)
        .response_3xx_count(req.counter.response_3xx_count + last.response_3xx_count)
        .response_4xx_count(req.counter.response_4xx_count + last.response_4xx_count)
        .response_5xx_count(req.counter.response_5xx_count + last.response_5xx_count)
        .avg_response_time(if req.counter.request_count > 0 {
            (req.counter.response_time_since_last / req.counter.request_count
                + last.avg_response_time)
                / 2
        } else {
            last.avg_response_time
        })
        .create_time(Some(tools::now()))
        .build()?;

    GatewayNodeState::insert(tx, &gateway_state_log).await?;

    Ok(())
}

fn send_alert(node: &GatewayNode) {
    Alert::info(
        "网关节点上线",
        &format!(
            "节点地址: {}:{}",
            node.ip.as_ref().unwrap(),
            node.port.as_ref().unwrap()
        ),
    );
}
