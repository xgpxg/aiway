use crate::server::db::Pool;
use crate::server::db::models::gateway_state::GatewayState;

pub async fn gateway_state() -> anyhow::Result<GatewayState> {
    // 仅有1条
    let state = GatewayState::select_all(Pool::get()?).await?;
    let state = state.first().unwrap().clone();
    Ok(state)
}
