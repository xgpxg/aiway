use crate::config::RaftApp;
use crate::config::raft::declare_types::{
    AppendEntriesRequest, InstallSnapshotRequest, VoteRequest,
};
use openraft::error::{InstallSnapshotError, RaftError};
use openraft::raft::AppendEntriesResponse;
use openraft::raft::InstallSnapshotResponse;
use openraft::raft::VoteResponse;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{State, post};

#[post("/vote", data = "<req>")]
pub async fn vote(
    app: &State<RaftApp>,
    req: Json<VoteRequest>,
) -> Result<Json<VoteResponse<u64>>, Status> {
    match app.raft.vote(req.into_inner()).await {
        Ok(response) => Ok(Json(response)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/append", data = "<req>")]
pub async fn append(
    app: &State<RaftApp>,
    req: Json<AppendEntriesRequest>,
) -> Result<Json<AppendEntriesResponse<u64>>, Status> {
    match app.raft.append_entries(req.0).await {
        Ok(response) => Ok(Json(response)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/snapshot", data = "<req>")]
pub async fn snapshot(
    app: &State<RaftApp>,
    req: Json<InstallSnapshotRequest>,
) -> Result<Json<InstallSnapshotResponse<u64>>, Status> {
    match app.raft.install_snapshot(req.0).await {
        Ok(response) => Ok(Json(response)),
        Err(_) => Err(Status::InternalServerError),
    }
}
