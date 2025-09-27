use rocket::form::FromForm;
use rocket::fs::TempFile;
use protocol::gateway::plugin::PluginPhase;

#[derive(Debug, FromForm)]
pub struct PluginAddOrUpdateReq<'a> {
    pub name: String,
    pub description: String,
    pub version: String,
    pub phase: PluginPhase,
    pub file: TempFile<'a>,
}
