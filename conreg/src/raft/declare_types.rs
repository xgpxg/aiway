use crate::raft::{NodeId, TypeConfig};
use openraft::BasicNode;

pub type LogId = openraft::LogId<NodeId>;
pub type Entry = <TypeConfig as openraft::RaftTypeConfig>::Entry;
pub type EntryPayload = openraft::EntryPayload<TypeConfig>;
pub type StoredMembership = openraft::StoredMembership<NodeId, BasicNode>;
pub type Node = <TypeConfig as openraft::RaftTypeConfig>::Node;
pub type SnapshotMeta = openraft::SnapshotMeta<NodeId, BasicNode>;
pub type SnapshotData = <TypeConfig as openraft::RaftTypeConfig>::SnapshotData;
pub type StorageError = openraft::StorageError<NodeId>;
pub type VoteRequest = openraft::raft::VoteRequest<NodeId>;
pub type ClientWriteResponse = openraft::raft::ClientWriteResponse<TypeConfig>;
pub type RaftMetrics = openraft::metrics::RaftMetrics<NodeId, BasicNode>;
