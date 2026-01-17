use crate::server::db::Pool;
use crate::server::db::models::{gateway_node, gateway_node_state};
use crate::server::node::request::GatewayNodeListReq;
use crate::server::node::response::GatewayNodeListRes;
use busi::req::Pagination;
use busi::res::{IntoPageRes, PageRes};

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
