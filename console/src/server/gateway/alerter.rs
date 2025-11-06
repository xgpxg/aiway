use alert::pusher::Pusher;
use protocol::gateway::alert::{AlertConfig, AlertMessage};

pub(crate) async fn alert(req: AlertMessage) -> anyhow::Result<()> {
    println!("alert: {:?}", req);

    let mut config = AlertConfig::default();
    config.dingding.enable = true;
    config.dingding.webhook = "webhook地址".to_string();
    Pusher::push(config.into(), req);

    Ok(())
}
