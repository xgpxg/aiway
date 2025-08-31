pub mod sled_log_store;

use crate::config::raft::declare_types::{
    Entry, EntryPayload, LogId, SnapshotData, SnapshotMeta, StorageError, StoredMembership,
};
use crate::config::raft::{NodeId, RaftRequest, RaftResponse, TypeConfig};
use openraft::entry::RaftEntry;
use openraft::storage::RaftStateMachine;
use openraft::storage::Snapshot;
use openraft::{AnyError, RaftSnapshotBuilder, RaftTypeConfig, StorageIOError, add_async_trait};
use rand::Rng;
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
use logging::log;

#[derive(Serialize, Deserialize, Debug)]
pub struct StoredSnapshot {
    /// 快照元数据
    pub meta: SnapshotMeta,
    /// 快照数据
    pub data: Vec<u8>,
}

/// 定义状态机数据
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StateMachineData {
    /// 当前已处理的日志ID
    pub last_applied_log: Option<LogId>,
    /// 记录当前状态机所知道的最新集群成员配置
    pub last_membership: StoredMembership,
    /// 应用数据
    pub data: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct StateMachineStore {
    /// 当前状态机数据
    pub data: Arc<RwLock<StateMachineData>>,
    /// 快照索引，一般使用自增或者当前微秒时间戳即可
    pub snapshot_idx: u64,
    /// 数据库
    pub db: Arc<DB>,
}
impl StateMachineStore {
    async fn new(db: Arc<DB>) -> StateMachineStore {
        let mut state_machine = Self {
            data: Default::default(),
            snapshot_idx: 0,
            db,
        };

        log::info!("load state machine from db");

        // 加载状态机最新快照
        let snapshot = state_machine.get_current_snapshot().await.unwrap();

        // 从快照中恢复状态机
        if let Some(s) = snapshot {
            let prev: StateMachineData = serde_json::from_slice(s.snapshot.get_ref()).unwrap();
            state_machine.data = Arc::new(RwLock::new(prev));
        }

        state_machine
    }
}

/// 实现快照
impl RaftSnapshotBuilder<TypeConfig> for StateMachineStore {
    async fn build_snapshot(&mut self) -> Result<Snapshot<TypeConfig>, StorageError> {
        let data_write_guard = self.data.write().await;

        // 序列化状态机
        let data =
            serde_json::to_vec(data_write_guard.deref()).map_err(|e| StorageIOError::read_state_machine(&e))?;

        let last_applied_log = data_write_guard.last_applied_log;
        let last_membership = data_write_guard.last_membership.clone();

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

    /// 获取状态机中最新的applied的log_id和最新的集群成员信息
    async fn applied_state(&mut self) -> Result<(Option<LogId>, StoredMembership), StorageError> {
        println!(
            "applied_state: {:?}",
            (
                self.data.read().await.last_applied_log,
                self.data.read().await.last_membership.clone(),
            )
        );
        Ok((
            self.data.read().await.last_applied_log,
            self.data.read().await.last_membership.clone(),
        ))
    }

    async fn apply<I>(&mut self, entries: I) -> Result<Vec<RaftResponse>, StorageError>
    where
        I: IntoIterator<Item = Entry> + Send,
    {
        let entries_iter = entries.into_iter();
        let mut res = Vec::with_capacity(entries_iter.size_hint().0);

        let data = &mut self.data;

        let mut data_write_guard = data.write().await;
        for entry in entries_iter {
            data_write_guard.last_applied_log = Some(entry.log_id);

            match entry.payload {
                EntryPayload::Blank => res.push(RaftResponse { value: None }),
                EntryPayload::Normal(ref req) => match req {
                    RaftRequest::Set { key, value } => {
                        data_write_guard.data.insert(key.clone(), value.clone());
                        res.push(RaftResponse {
                            value: Some(value.clone()),
                        });
                    }
                },
                EntryPayload::Membership(ref mem) => {
                    data_write_guard.last_membership = StoredMembership::new(Some(entry.log_id), mem.clone());
                    res.push(RaftResponse { value: None })
                }
            };
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

        self.data = Arc::new(RwLock::new(updated_state_machine));

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

    async fn get_current_snapshot(&mut self) -> Result<Option<Snapshot<TypeConfig>>, StorageError> {
        // 使用 sled 读取快照
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
/// Create a pair of `RocksLogStore` and `RocksStateMachine` that are backed by a same rocks db
/// instance.
pub async fn new<C, P: AsRef<Path>>(db_path: P) -> (SledLogStore<C>, StateMachineStore)
where
    C: RaftTypeConfig,
{
    // 创建 sled 数据库配置
    let mut db_config = sled::Config::new();
    db_config = db_config.path(db_path);

    // 打开数据库
    let db = Arc::new(db_config.open().expect("Failed to open sled database"));

    db.open_tree("meta").expect("Failed to create meta tree");
    db.open_tree("sm_meta")
        .expect("Failed to create sm_meta tree");
    db.open_tree("logs").expect("Failed to create logs tree");

    (
        SledLogStore::new(db.clone()),
        StateMachineStore::new(db).await,
    )
}
