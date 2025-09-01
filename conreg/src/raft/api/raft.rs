use crate::raft::declare_types::VoteRequest;
use crate::raft::{NodeId, TypeConfig};
use logging::log;
use openraft::error::RaftError;
use openraft::raft::InstallSnapshotResponse;
use openraft::raft::VoteResponse;
use openraft::raft::{AppendEntriesRequest, AppendEntriesResponse, InstallSnapshotRequest};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{State, post};
use crate::app::App;

#[post("/vote", data = "<req>")]
pub async fn vote(
    app: &State<App>,
    req: Json<VoteRequest>,
) -> Result<Json<Result<VoteResponse<NodeId>, RaftError<NodeId>>>, Status> {
    match app.raft.vote(req.into_inner()).await {
        Ok(response) => Ok(Json(Ok(response))),
        Err(e) => {
            log::error!("Vote error: {}", e);
            Err(Status::InternalServerError)
        }
    }
}

/// 当需要同步日志或者心跳时触发调用。
/// 当为心跳请求时，entries为空数组。
///
/// 整体流程：
/// 1. 客户端提交写请求
/// 2. Leader将日志追加到本地
/// 3. Leader向所有Follower发送 AppendEntries RPC
/// 4. Follower的 /append 接口被调用
#[post("/append", data = "<req>")]
pub async fn append(
    app: &State<App>,
    req: Json<AppendEntriesRequest<TypeConfig>>,
) -> Result<Json<Result<AppendEntriesResponse<NodeId>, RaftError<NodeId>>>, Status> {
    match app.raft.append_entries(req.0).await {
        Ok(response) => Ok(Json(Ok(response))),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/snapshot", data = "<req>")]
pub async fn snapshot(
    app: &State<App>,
    req: Json<InstallSnapshotRequest<TypeConfig>>,
) -> Result<Json<Result<InstallSnapshotResponse<NodeId>, RaftError<NodeId>>>, Status> {
    match app.raft.install_snapshot(req.0).await {
        Ok(response) => Ok(Json(Ok(response))),
        Err(_) => Err(Status::InternalServerError),
    }
}
