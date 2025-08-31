use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::config::RaftApp;
use crate::config::raft::declare_types::{Node, RaftMetrics};
use crate::config::raft::{NodeId, TypeConfig};
use openraft::raft::ClientWriteResponse;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{State, get, post};
use logging::log;

/// Add a node as **Learner**.
///
/// A Learner receives log replication from the leader but does not vote.
/// This should be done before adding a node as a member into the cluster
/// (by calling `change-membership`)
#[post("/add-learner", data = "<req>")]
pub async fn add_learner(
    app: &State<RaftApp>,
    req: Json<(NodeId, String)>,
) -> Result<Json<ClientWriteResponse<TypeConfig>>, Status> {
    let (node_id, api_addr) = req.0;
    let node = Node { addr: api_addr };
    match app.raft.add_learner(node_id, node, true).await {
        Ok(response) => Ok(Json(response)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// Changes specified learners to members, or remove members.
#[post("/change-membership", data = "<req>")]
pub async fn change_membership(
    app: &State<RaftApp>,
    req: Json<BTreeSet<NodeId>>,
) -> Result<Json<ClientWriteResponse<TypeConfig>>, Status> {
    match app.raft.change_membership(req.0, false).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// Initialize a cluster.
#[post("/init", data = "<req>")]
pub async fn init(
    app: &State<RaftApp>,
    req: Json<Vec<(NodeId, String)>>,
) -> Result<Json<()>, Status> {
    let mut nodes = BTreeMap::new();
    if req.0.is_empty() {
        nodes.insert(
            app.id,
            Node {
                addr: app.addr.clone(),
            },
        );
    } else {
        for (id, addr) in req.0.into_iter() {
            nodes.insert(id, Node { addr });
        }
    };
    match app.raft.initialize(nodes).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            log::error!("{}", e);
            Err(Status::InternalServerError)
        }
    }
}

/// Get the latest metrics of the cluster
#[get("/metrics")]
pub async fn metrics(app: &State<RaftApp>) -> Result<Json<RaftMetrics>, Status> {
    let metrics = app.raft.metrics().borrow().clone();
    Ok(Json(metrics))
}
