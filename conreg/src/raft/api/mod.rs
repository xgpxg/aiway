use crate::raft::declare_types::ClientWriteResponse;
use crate::raft::{NodeId, RaftRequest};
use logging::log;
use rocket::http::Status;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

mod app;
mod manage;
mod raft;

pub fn routes() -> Vec<rocket::Route> {
    routes![
        raft::vote,
        raft::append,
        raft::snapshot,
        manage::init,
        manage::metrics,
        manage::change_membership,
        manage::add_learner,
        app::read,
        app::write,
    ]
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum ForwardRequest {
    RaftRequest(RaftRequest),
    MembershipRequest(BTreeSet<NodeId>),
}

async fn forward_request_to_leader(
    leader_addr: &str,
    request: ForwardRequest,
) -> Result<Json<ClientWriteResponse>, Status> {
    let client = reqwest::Client::new();

    let forward_url = match &request {
        ForwardRequest::RaftRequest(_) => {
            format!("http://{}/write", leader_addr)
        }
        ForwardRequest::MembershipRequest(_) => {
            format!("http://{}/change-membership", leader_addr)
        }
    };
    match client.post(&forward_url).json(&request).send().await {
        Ok(response) => match response.json::<ClientWriteResponse>().await {
            Ok(result) => Ok(Json(result)),
            Err(e) => {
                log::error!("Failed to parse forwarded response: {}", e);
                Err(Status::InternalServerError)
            }
        },
        Err(e) => {
            log::error!("Failed to forward request to leader {}: {}", leader_addr, e);
            Err(Status::ServiceUnavailable)
        }
    }
}
