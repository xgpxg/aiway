use crate::config::raft::{NodeId, TypeConfig};
use openraft::BasicNode;

pub type Raft = openraft::Raft<TypeConfig>;

pub type LogId = openraft::LogId<NodeId>;
pub type Entry = <TypeConfig as openraft::RaftTypeConfig>::Entry;
pub type EntryPayload = openraft::EntryPayload<TypeConfig>;
pub type Membership = openraft::Membership<BasicNode, NodeId>;
pub type StoredMembership = openraft::StoredMembership<NodeId, BasicNode>;

pub type Node = <TypeConfig as openraft::RaftTypeConfig>::Node;

pub type LogState = openraft::storage::LogState<TypeConfig>;

pub type SnapshotMeta = openraft::SnapshotMeta<NodeId, BasicNode>;
pub type Snapshot = openraft::Snapshot<TypeConfig>;
pub type SnapshotData = <TypeConfig as openraft::RaftTypeConfig>::SnapshotData;

pub type Infallible = openraft::error::Infallible;
pub type Fatal = openraft::error::Fatal<TypeConfig>;
pub type RaftError<E = openraft::error::Infallible> = openraft::error::RaftError<TypeConfig, E>;
pub type RPCError<E = openraft::error::Infallible> = openraft::error::RPCError<TypeConfig, E>;

pub type ErrorSubject = openraft::ErrorSubject<TypeConfig>;
pub type StorageError = openraft::StorageError<NodeId>;

pub type StreamingError = openraft::error::StreamingError<TypeConfig>;
pub type VoteRequest = openraft::raft::VoteRequest<NodeId>;
pub type VoteResponse = openraft::raft::VoteResponse<TypeConfig>;
pub type AppendEntriesRequest = openraft::raft::AppendEntriesRequest<TypeConfig>;
pub type AppendEntriesResponse = openraft::raft::AppendEntriesResponse<TypeConfig>;
pub type InstallSnapshotRequest = openraft::raft::InstallSnapshotRequest<TypeConfig>;
pub type InstallSnapshotResponse = openraft::raft::InstallSnapshotResponse<TypeConfig>;
pub type SnapshotResponse = openraft::raft::SnapshotResponse<TypeConfig>;
pub type ClientWriteResponse = openraft::raft::ClientWriteResponse<TypeConfig>;

pub type LogApplied = openraft::storage::LogApplied<TypeConfig>;
pub type RaftMetrics = openraft::metrics::RaftMetrics<NodeId, BasicNode>;
