use protocol::gateway::Firewall;
use rocket::serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct FirewallUpdateReq {
    #[serde(flatten)]
    pub inner: Firewall,
}
