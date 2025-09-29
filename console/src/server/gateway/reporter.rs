use logging::log;
use protocol::gateway::state::State;

/// 接收gateway上报数据
pub(crate) async fn report(req: State) -> anyhow::Result<()> {
    log::info!("{:?}", req);

    // 监控数据处理

    // 更新活跃状态

    Ok(())
}
