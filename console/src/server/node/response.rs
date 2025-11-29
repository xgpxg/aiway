use crate::server::db::models::gateway_node::GatewayNode;
use crate::server::db::models::gateway_node_state::GatewayNodeState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayNodeListRes {
    #[serde(flatten)]
    pub inner: GatewayNode,
    pub state: Option<GatewayNodeState>,
}
