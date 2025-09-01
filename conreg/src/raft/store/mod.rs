pub mod sled_log_store;

use crate::event::Event;
use crate::raft::declare_types::{
    Entry, EntryPayload, LogId, SnapshotData, SnapshotMeta, StorageError, StoredMembership,
};
use crate::raft::{NodeId, RaftRequest, RaftResponse, TypeConfig};
use logging::log;
use openraft::storage::RaftStateMachine;
use openraft::storage::Snapshot;
use openraft::{AnyError, RaftSnapshotBuilder, RaftTypeConfig, StorageIOError};
use rocket::yansi::Paint;
use serde::Deserialize;
use serde::Serialize;
use sled::Db as DB;
pub(crate) use sled_log_store::SledLogStore;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::io::Cursor;
use std::ops::Deref;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::mpsc::error::SendError;

#[derive(Serialize, Deserialize, Debug)]
pub struct StoredSnapshot {
    /// 快照元数据
    pub meta: SnapshotMeta,
    /// 快照数据，这里即StateMachineData序列化
    pub data: Vec<u8>,
}

/// 定义状态机数据
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StateMachineData {
    /// 当前已处理的日志ID
    pub last_applied_log: Option<LogId>,
    /// 记录当前状态机所知道的最新集群成员配置
    pub last_membership: StoredMembership,
    /// 应用数据，仅用做简单KV存储。
    pub data: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct StateMachineStore {
    /// 当前状态机数据
    pub state_machine: Arc<RwLock<StateMachineData>>,
    /// 快照索引，一般使用自增或者当前微秒时间戳即可
    pub snapshot_idx: u64,
    /// KV库，用于存储序列化后的状态机快照
    pub db: Arc<DB>,
}

impl StateMachineStore {
    async fn new(db: Arc<DB>) -> StateMachineStore {
        let mut state_machine = Self {
            state_machine: Default::default(),
            snapshot_idx: 0,
            db,
        };

        log::info!("load state machine from db");

        // 加载状态机最新快照
        let snapshot = state_machine.get_current_snapshot().await.unwrap();

        // 从快照中恢复状态机
        if let Some(s) = snapshot {
            let prev: StateMachineData = serde_json::from_slice(s.snapshot.get_ref()).unwrap();
            state_machine.state_machine = Arc::new(RwLock::new(prev));
        }

        state_machine
    }

    /// 应用每一个日志条目
    async fn apply_entry(self: &mut Self, entry: Entry) -> Result<RaftResponse, StorageError> {
        let mut state_machine = self.state_machine.write().await;

        // 更新last_applied_log，注意这里没有持久化，而是等待日志条目数量达到一定值时触发状态机持久化时才会持久化状态。
        // 也就是说，如果节点重启，则内存中的last_applied_log丢失，将从磁盘恢复到上一次快照时的状态机，
        // 然后会重新应用从快照点到最新的日志条目。
        state_machine.last_applied_log = Some(entry.log_id);

        // 业务处理
        // TODO 可能的问题：
        // 1. 目前均按照成功处理，处理失败时打印日志，可能会导致部分处理失败的被跳过
        // 2. SetConfig以Event的方式处理，无法获取结果，如果Event处理失败，可能会导致数据丢失。
        match entry.payload {
            EntryPayload::Blank => Ok(RaftResponse { value: None }),
            EntryPayload::Normal(ref req) => match req {
                RaftRequest::Set { key, value } => {
                    state_machine.data.insert(key.clone(), value.clone());
                    Ok(RaftResponse {
                        value: Some(value.clone()),
                    })
                }
                RaftRequest::Delete { key } => {
                    let old = state_machine.data.remove(key);
                    Ok(RaftResponse { value: old })
                }
                // 处理配置中心的配置变更操作
                RaftRequest::SetConfig { .. } | RaftRequest::DeleteConfig { .. } => {
                    match Event::RaftRequestEvent(req.clone()).send() {
                        Ok(_) => Ok(RaftResponse { value: None }),
                        Err(e) => {
                            log::error!("Failed to send SetConfig event: {:?}", e);
                            Err(StorageIOError::write_state_machine(AnyError::new(&e)).into())
                        }
                    }
                }
            },
            EntryPayload::Membership(ref mem) => {
                state_machine.last_membership =
                    StoredMembership::new(Some(entry.log_id), mem.clone());
                Ok(RaftResponse { value: None })
            }
        }
    }
}

/// 实现快照
///
/// 这里的快照仅仅是对状态机的持久化（包含状态机内部的KV数据）
impl RaftSnapshotBuilder<TypeConfig> for StateMachineStore {
    /// 生成快照
    async fn build_snapshot(&mut self) -> Result<Snapshot<TypeConfig>, StorageError> {
        let state_machine = self.state_machine.write().await;

        // 序列化状态机
        let data = serde_json::to_vec(state_machine.deref())
            .map_err(|e| StorageIOError::read_state_machine(&e))?;

        let last_applied_log = state_machine.last_applied_log;
        let last_membership = state_machine.last_membership.clone();

        // 唯一的快照ID
        let snapshot_id = if let Some(last) = last_applied_log {
            format!(
                "{}-{}-{}",
                last.committed_leader_id(),
                last.index,
                self.snapshot_idx
            )
        } else {
            format!("--{}", self.snapshot_idx)
        };

        // 快照元数据
        let meta = SnapshotMeta {
            last_log_id: last_applied_log,
            last_membership,
            snapshot_id,
        };

        // 快照数据
        let snapshot = StoredSnapshot {
            meta: meta.clone(),
            data: data.clone(),
        };

        // 序列化
        let serialized_snapshot = serde_json::to_vec(&snapshot).map_err(|e| {
            StorageIOError::write_snapshot(Some(meta.signature()), AnyError::new(&e))
        })?;

        // 使用 sled 存储快照
        let sm_meta_tree = self.db.open_tree("sm_meta").map_err(|e| {
            StorageIOError::write_snapshot(Some(meta.signature()), AnyError::new(&e))
        })?;

        sm_meta_tree
            .insert("snapshot", serialized_snapshot)
            .map_err(|e| {
                StorageIOError::write_snapshot(Some(meta.signature()), AnyError::new(&e))
            })?;

        sm_meta_tree.flush_async().await.map_err(|e| {
            StorageIOError::write_snapshot(Some(meta.signature()), AnyError::new(&e))
        })?;

        Ok(Snapshot {
            meta,
            snapshot: Box::new(Cursor::new(data)),
        })
    }
}

/// 实现Raft状态机
impl RaftStateMachine<TypeConfig> for StateMachineStore {
    type SnapshotBuilder = Self;

    /// 获取状态机中获取最新的applied的log_id和最新的集群成员信息
    async fn applied_state(&mut self) -> Result<(Option<LogId>, StoredMembership), StorageError> {
        let state = (
            self.state_machine.read().await.last_applied_log,
            self.state_machine.read().await.last_membership.clone(),
        );
        log::info!("applied state of last applied log id: {:?}", state.0);
        log::info!("applied state of last membership: {:?}", state.1);
        Ok((state.0, state.1))
    }

    /// 应用日志条目
    ///
    /// 该方法要么在应用每一个日志条目时完成对last_applied_log的持久化，要么持久化快照，以便可以正确恢复。
    /// 如果使用持久化状态机快照，则在恢复时，raft会自动重新apply快照之后的日志条目，因此，应该保证业务逻辑的幂等性。
    /// 目前使用持久化状态机快照的方式
    async fn apply<I>(&mut self, entries: I) -> Result<Vec<RaftResponse>, StorageError>
    where
        I: IntoIterator<Item = Entry> + Send,
    {
        // 需要处理的日志条目
        let entries_iter = entries.into_iter();
        let mut res = Vec::with_capacity(entries_iter.size_hint().0);

        for entry in entries_iter {
            log::info!("apply entry: {:?}", entry);
            res.push(self.apply_entry(entry).await?);
        }
        Ok(res)
    }

    async fn get_snapshot_builder(&mut self) -> Self::SnapshotBuilder {
        self.snapshot_idx += 1;
        self.clone()
    }

    async fn begin_receiving_snapshot(
        &mut self,
    ) -> Result<Box<SnapshotData>, openraft::StorageError<NodeId>> {
        Ok(Box::new(Cursor::new(Vec::new())))
    }

    async fn install_snapshot(
        &mut self,
        meta: &SnapshotMeta,
        snapshot: Box<SnapshotData>,
    ) -> Result<(), StorageError> {
        tracing::info!(
            { snapshot_size = snapshot.get_ref().len() },
            "decoding snapshot for installation"
        );

        let new_snapshot = StoredSnapshot {
            meta: meta.clone(),
            data: snapshot.into_inner(),
        };

        // Update the state machine.
        let updated_state_machine: StateMachineData = serde_json::from_slice(&new_snapshot.data)
            .map_err(|e| StorageIOError::read_snapshot(Some(new_snapshot.meta.signature()), &e))?;

        self.state_machine = Arc::new(RwLock::new(updated_state_machine));

        // Save snapshot using sled
        let serialized_snapshot = serde_json::to_vec(&new_snapshot).map_err(|e| {
            StorageIOError::write_snapshot(Some(meta.signature()), AnyError::new(&e))
        })?;

        let sm_meta_tree = self.db.open_tree("sm_meta").map_err(|e| {
            StorageIOError::write_snapshot(Some(meta.signature()), AnyError::new(&e))
        })?;

        sm_meta_tree
            .insert("snapshot", serialized_snapshot)
            .map_err(|e| {
                StorageIOError::write_snapshot(Some(meta.signature()), AnyError::new(&e))
            })?;

        sm_meta_tree.flush_async().await.map_err(|e| {
            StorageIOError::write_snapshot(Some(meta.signature()), AnyError::new(&e))
        })?;

        Ok(())
    }

    /// 获取当前快照
    ///
    /// 该快照包含2部分：
    /// - 元数据：元数据包含了last_log_id和last_membership
    /// - 快照数据
    /// 重启时可通过次快照恢复
    async fn get_current_snapshot(&mut self) -> Result<Option<Snapshot<TypeConfig>>, StorageError> {
        let sm_meta_tree = self
            .db
            .open_tree("sm_meta")
            .map_err(|e| StorageIOError::write_snapshot(None, AnyError::new(&e)))?;

        let bytes = sm_meta_tree
            .get("snapshot")
            .map_err(|e| StorageIOError::write_snapshot(None, AnyError::new(&e)))?;

        let bytes = match bytes {
            Some(x) => x,
            None => return Ok(None),
        };

        let snapshot: StoredSnapshot = serde_json::from_slice(&bytes)
            .map_err(|e| StorageIOError::write_snapshot(None, AnyError::new(&e)))?;

        let data = snapshot.data.clone();

        Ok(Some(Snapshot {
            meta: snapshot.meta,
            snapshot: Box::new(Cursor::new(data)),
        }))
    }
}

pub async fn new<C, P: AsRef<Path>>(db_path: P) -> (SledLogStore<C>, StateMachineStore)
where
    C: RaftTypeConfig,
{
    let mut cfg = sled::Config::new();
    cfg = cfg.path(format!("{}/raft", db_path.as_ref().display()));

    let db = Arc::new(cfg.open().expect("Failed to open sled database"));

    // 元数据
    db.open_tree("meta").expect("Failed to create meta tree");
    // 状态机元数据
    db.open_tree("sm_meta")
        .expect("Failed to create sm_meta tree");
    // 日志
    db.open_tree("logs").expect("Failed to create logs tree");

    (
        SledLogStore::new(db.clone()),
        StateMachineStore::new(db).await,
    )
}
