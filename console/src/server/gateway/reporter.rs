use crate::server::db::Pool;
use crate::server::db::models::gateway_node_state_log::{
    GatewayNodeStateLog, GatewayNodeStateLogBuilder,
};
use common::id;
use logging::log;
use protocol::gateway::state::State;

/// 接收gateway上报数据
pub async fn report(node_id: String, req: State) -> anyhow::Result<()> {
    log::info!("node_id:{}, state: {:?}", node_id, req);

    // 监控数据处理
    let gateway_state_log = GatewayNodeStateLogBuilder::default()
        .id(Some(id::next()))
        .node_id(Some(node_id))
        .ts(Some(req.timestamp))
        .os(Some(req.system_state.os))
        .cpu_usage(Some(req.system_state.cpu_usage))
        .mem_total(Some(req.system_state.mem_state.total))
        .mem_free(Some(req.system_state.mem_state.free))
        .mem_used(Some(req.system_state.mem_state.used))
        .disk_total(Some(req.system_state.disk_state.total))
        .disk_free(Some(req.system_state.disk_state.free))
        .net_rx(Some(req.system_state.net_state.rx))
        .net_tx(Some(req.system_state.net_state.tx))
        .net_tcp_conn_count(Some(req.system_state.net_state.tcp_conn_count))
        .request_count(Some(req.counter.request_count))
        .request_invalid_count(Some(req.counter.request_invalid_count))
        .response_2xx_count(Some(req.counter.response_2xx_count))
        .response_3xx_count(Some(req.counter.response_3xx_count))
        .response_4xx_count(Some(req.counter.response_4xx_count))
        .response_5xx_count(Some(req.counter.response_5xx_count))
        .build()?;

    let tx = Pool::get()?;
    GatewayNodeStateLog::insert(tx, &gateway_state_log).await?;

    // 更新活跃状态

    Ok(())
}
