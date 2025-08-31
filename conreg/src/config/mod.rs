use crate::config::raft::NodeId;

pub mod raft;

pub use raft::App as RaftApp;

pub async fn new_raft_app(node_id: NodeId) -> RaftApp {
    raft::new_raft_app(node_id).await
}


