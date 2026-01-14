use aiway_protocol::gateway::Config;

pub(crate) async fn config() -> anyhow::Result<Config> {
    let config = Config {
        // 暂时没有配置
    };
    Ok(config)
}
