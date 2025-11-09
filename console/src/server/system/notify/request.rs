use protocol::gateway::alert::AlertConfig;
use rocket::serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct NotifyConfigUpdateReq {
    #[serde(flatten)]
    pub inner:AlertConfig,
}
