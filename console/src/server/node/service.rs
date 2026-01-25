use crate::server::db::Pool;
use crate::server::db::models::{gateway_node, gateway_node_state};
use crate::server::node::request::GatewayNodeListReq;
use crate::server::node::response::{GatewayNodeListRes, UsageRes};
use busi::req::Pagination;
use busi::res::{IntoPageRes, PageRes};
use rbs::value;

pub(crate) async fn list(req: GatewayNodeListReq) -> anyhow::Result<PageRes<GatewayNodeListRes>> {
    let tx = Pool::get()?;

    let page = gateway_node::list_page(tx, &req.to_rb_page(), &req).await?;

    let node_ids = page
        .records
        .iter()
        .map(|item| item.node_id.clone().unwrap_or_default())
        .collect::<Vec<_>>();

    let states = if node_ids.is_empty() {
        vec![]
    } else {
        gateway_node_state::lastest_state(tx, &node_ids).await?
    };

    let states_map = states
        .into_iter()
        .map(|item| (item.node_id.clone(), item))
        .collect::<std::collections::HashMap<_, _>>();

    let list = page.convert_to_page_res(|list| {
        list.into_iter()
            .map(|item| {
                let state = states_map.get(&item.node_id.clone().unwrap_or_default());
                GatewayNodeListRes {
                    inner: item,
                    state: state.cloned(),
                }
            })
            .collect::<Vec<_>>()
    });
    Ok(list)
}

pub(crate) async fn node_cpu_usage(
    node_id: &str,
    start_timestamp: i64,
    end_timestamp: i64,
) -> anyhow::Result<Vec<UsageRes>> {
    let tx = Pool::get()?;
    let list: Vec<UsageRes> = tx.query_decode(
        "SELECT ts as t,cpu_usage as v  FROM gateway_node_state  WHERE node_id = ? AND ts >= ? AND ts <= ? ORDER BY ts ASC LIMIT 20000",
        vec![
            value!(node_id),
            value!(start_timestamp),
            value!(end_timestamp),
        ],
    )
        .await?;
    Ok(list)
}

pub(crate) async fn node_memory_usage(
    node_id: &str,
    start_timestamp: i64,
    end_timestamp: i64,
) -> anyhow::Result<Vec<UsageRes>> {
    let tx = Pool::get()?;
    let list: Vec<UsageRes> = tx.query_decode(
        "SELECT ts as t,mem_used as v  FROM gateway_node_state  WHERE node_id = ? AND ts >= ? AND ts <= ? ORDER BY ts ASC LIMIT 20000",
        vec![
            value!(node_id),
            value!(start_timestamp),
            value!(end_timestamp),
        ],
    )
        .await?;
    Ok(list)
}

pub(crate) async fn node_network_usage(
    node_id: &str,
    start_timestamp: i64,
    end_timestamp: i64,
) -> anyhow::Result<(Vec<UsageRes>, Vec<UsageRes>)> {
    let tx = Pool::get()?;
    let rx_list: Vec<UsageRes> = tx.query_decode(
        "SELECT ts as t,net_rx as v  FROM gateway_node_state  WHERE node_id = ? AND ts >= ? AND ts <= ? ORDER BY ts ASC LIMIT 20000",
        vec![
            value!(node_id),
            value!(start_timestamp),
            value!(end_timestamp),
        ],
    )
        .await?;
    let tx_list: Vec<UsageRes> = tx.query_decode(
        "SELECT ts as t,net_tx as v  FROM gateway_node_state  WHERE node_id = ? AND ts >= ? AND ts <= ? ORDER BY ts ASC LIMIT 20000",
        vec![
            value!(node_id),
            value!(start_timestamp),
            value!(end_timestamp),
        ],
    )
        .await?;

    Ok((rx_list, tx_list))
}

pub(crate) async fn node_connection_usage(
    node_id: &str,
    start_timestamp: i64,
    end_timestamp: i64,
) -> anyhow::Result<(Vec<UsageRes>, Vec<UsageRes>, Vec<UsageRes>)> {
    let tx = Pool::get()?;
    let tcp_list: Vec<UsageRes> = tx.query_decode(
        "SELECT ts as t,net_tcp_conn_count as v  FROM gateway_node_state  WHERE node_id = ? AND ts >= ? AND ts <= ? ORDER BY ts ASC LIMIT 20000",
        vec![
            value!(node_id),
            value!(start_timestamp),
            value!(end_timestamp),
        ],
    )
        .await?;
    let http_list: Vec<UsageRes> = tx.query_decode(
        "SELECT ts as t,http_connect_count as v  FROM gateway_node_state  WHERE node_id = ? AND ts >= ? AND ts <= ? ORDER BY ts ASC LIMIT 20000",
        vec![
            value!(node_id),
            value!(start_timestamp),
            value!(end_timestamp),
        ],
    )
        .await?;
    let sse_list: Vec<UsageRes> = tx.query_decode(
        "SELECT ts as t,sse_connect_count as v  FROM gateway_node_state  WHERE node_id = ? AND ts >= ? AND ts <= ? ORDER BY ts ASC LIMIT 20000",
        vec![
            value!(node_id),
            value!(start_timestamp),
            value!(end_timestamp),
        ],
    )
        .await?;
    Ok((tcp_list, http_list, sse_list))
}
