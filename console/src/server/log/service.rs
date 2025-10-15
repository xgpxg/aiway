use protocol::common::res::PageRes;
use crate::server::log::request::LogListReq;
use crate::server::log::response::LogListRes;

pub async fn list(req: LogListReq) -> anyhow::Result<PageRes<LogListRes>> {
    todo!()
}
