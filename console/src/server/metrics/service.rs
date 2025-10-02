use crate::server::metrics::response::GatewayState;

pub async fn gateway_state() -> anyhow::Result<GatewayState> {
    let state = GatewayState::default();
    Ok(state)
}
