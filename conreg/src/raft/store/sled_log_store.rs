use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::RangeBounds;
use std::sync::Arc;

use openraft::RaftTypeConfig;
use openraft::StorageError;
use openraft::storage::{LogFlushed, RaftLogStorage};
use openraft::{LogState, Vote};
use openraft::{OptionalSend, StorageIOError};
use openraft::{RaftLogId, RaftLogReader};
use sled::IVec;
use logging::log;

/// 基于Sled实现的日志存储。
///
/// 官方给了rocksdb的示例，但是考虑到需要跨平台，而sled完全使用rust实现，可能更合适一点，
/// 并且对于配置中心来说，是读多写少的场景，sled能满足需求。
///
///
#[derive(Debug, Clone)]
pub struct SledLogStore<C>
where
    C: RaftTypeConfig,
{
    /// sled数据库
    db: Arc<sled::Db>,
    /// 占位，保持对泛型C的使用
    _p: PhantomData<C>,
}

impl<C> SledLogStore<C>
where
    C: RaftTypeConfig,
{
    pub fn new(db: Arc<sled::Db>) -> Self {
        Self {
            db,
            _p: Default::default(),
        }
    }

    /// 获取日志树
    fn logs_tree(&self) -> sled::Tree {
        self.db.open_tree("logs").expect("Failed to open logs tree")
    }

    /// 获取元数据树
    fn meta_tree(&self) -> sled::Tree {
        self.db.open_tree("meta").expect("Failed to open meta tree")
    }

    /// 获取元数据
    fn get_meta(&self, key: &str) -> Result<Option<IVec>, sled::Error> {
        let tree = self.meta_tree();
        let value = tree.get(key)?;
        Ok(value)
    }

    /// 写入元数据
    fn put_meta(&self, key: &str, value: &[u8]) -> Result<(), sled::Error> {
        let tree = self.meta_tree();
        tree.insert(key, value)?;
        tree.flush()?;
        Ok(())
    }
}

impl<C> RaftLogReader<C> for SledLogStore<C>
where
    C: RaftTypeConfig,
{
    /// 获取指定范围内的日志
    async fn try_get_log_entries<RB: RangeBounds<u64> + Clone + Debug + OptionalSend>(
        &mut self,
        range: RB,
    ) -> Result<Vec<C::Entry>, StorageError<C::NodeId>> {
        // 日志树
        let tree = self.logs_tree();

        // 待返回的结果
        let mut res = Vec::new();

        let start = match range.start_bound() {
            std::ops::Bound::Included(x) => *x,
            std::ops::Bound::Excluded(x) => *x + 1,
            std::ops::Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            std::ops::Bound::Included(x) => Some(*x + 1),
            std::ops::Bound::Excluded(x) => Some(*x),
            std::ops::Bound::Unbounded => None,
        };

        // 获取范围内的log
        let iter = tree.range(start.to_be_bytes()..);
        for item in iter {
            let (key, val) = item.map_err(|e| StorageIOError::read_logs(&e))?;
            let id = u64::from_be_bytes(key[..8].try_into().map_err(|_| {
                StorageIOError::read_logs(&std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid key format",
                ))
            })?);

            if let Some(end_val) = end {
                if id >= end_val {
                    break;
                }
            }

            if !range.contains(&id) {
                continue;
            }

            let entry: C::Entry =
                serde_json::from_slice(&val).map_err(|e| StorageIOError::read_logs(&e))?;
            assert_eq!(id, entry.get_log_id().index);
            res.push(entry);
        }

        Ok(res)
    }
}

impl<C> RaftLogStorage<C> for SledLogStore<C>
where
    C: RaftTypeConfig,
{
    type LogReader = Self;

    /// 获取最后被清理的日志ID和最新的日志ID
    async fn get_log_state(&mut self) -> Result<LogState<C>, StorageError<C::NodeId>> {
        let tree = self.logs_tree();

        // Get last log id
        let last_log_id =
            if let Some((_, val)) = tree.last().map_err(|e| StorageIOError::read_logs(&e))? {
                let entry: C::Entry =
                    serde_json::from_slice(&val).map_err(|e| StorageIOError::read_logs(&e))?;
                Some(entry.get_log_id().clone())
            } else {
                None
            };

        // Get last purged log id
        let last_purged_log_id = if let Some(bytes) = self
            .get_meta("last_purged_log_id")
            .map_err(|e| StorageIOError::read_logs(&e))?
        {
            let log_id: Option<openraft::LogId<C::NodeId>> =
                serde_json::from_slice(&bytes).map_err(|e| StorageIOError::read_vote(&e))?;
            log_id
        } else {
            None
        };

        let last_log_id = last_log_id.or(last_purged_log_id.clone());

        Ok(LogState {
            last_purged_log_id,
            last_log_id,
        })
    }

    async fn get_log_reader(&mut self) -> Self::LogReader {
        self.clone()
    }

    /// 保存投票
    ///
    /// 必须持久化后才能返回
    async fn save_vote(&mut self, vote: &Vote<C::NodeId>) -> Result<(), StorageError<C::NodeId>> {
        let serialized = serde_json::to_vec(vote).map_err(|e| StorageIOError::write_vote(&e))?;
        self.put_meta("vote", &serialized)
            .map_err(|e| StorageIOError::write_vote(&e))?;
        Ok(())
    }

    /// 获取最新的投票信息
    async fn read_vote(&mut self) -> Result<Option<Vote<C::NodeId>>, StorageError<C::NodeId>> {
        if let Some(bytes) = self
            .get_meta("vote")
            .map_err(|e| StorageIOError::read_vote(&e))?
        {
            let vote = serde_json::from_slice(&bytes).map_err(|e| StorageIOError::read_vote(&e))?;
            Ok(Some(vote))
        } else {
            Ok(None)
        }
    }

    /// 追加日志
    ///
    /// 按照要求，应该在写入内存后立即返回，当完成持久化后调用callback，
    /// 但是按照注释，callback可以在本方法返回前或返回后调用，有点歧义，待确认。
    async fn append<I>(
        &mut self,
        entries: I,
        callback: LogFlushed<C>,
    ) -> Result<(), StorageError<C::NodeId>>
    where
        I: IntoIterator<Item = C::Entry> + Send,
    {
        let tree = self.logs_tree();
        for entry in entries {
            let id = entry.get_log_id().index;
            let serialized =
                serde_json::to_vec(&entry).map_err(|e| StorageIOError::write_logs(&e))?;
            tree.insert(&id.to_be_bytes(), serialized)
                .map_err(|e| StorageIOError::write_logs(&e))?;
        }

        tree.flush_async()
            .await
            .map_err(|e| StorageIOError::write_logs(&e))?;
        callback.log_io_completed(Ok(()));
        Ok(())
    }

    /// 从指定的 log_id 开始截断日志，包含该log_id
    async fn truncate(
        &mut self,
        log_id: openraft::LogId<C::NodeId>,
    ) -> Result<(), StorageError<C::NodeId>> {
        tracing::debug!("truncate: [{:?}, +oo)", log_id);
        let tree = self.logs_tree();
        //let start_key = log_id.index().to_be_bytes();

        // 从指定的log_id截断到最新的（包含该log_id）
        let mut batch = sled::Batch::default();
        for result in tree.range(log_id.get_log_id().index.to_be_bytes()..) {
            let (key, _) = result.map_err(|e| StorageIOError::write_logs(&e))?;
            batch.remove(key);
        }

        tree.apply_batch(batch)
            .map_err(|e| StorageIOError::write_logs(&e))?;
        tree.flush_async()
            .await
            .map_err(|e| StorageIOError::write_logs(&e))?;
        Ok(())
    }

    /// 清理从开始到指定log_id的日志，包含该log_id
    async fn purge(
        &mut self,
        log_id: openraft::LogId<C::NodeId>,
    ) -> Result<(), StorageError<C::NodeId>> {
        tracing::debug!("delete_log: [0, {:?}]", log_id);
        let tree = self.logs_tree();

        // Save the last purged log id
        let serialized = serde_json::to_vec(&log_id).map_err(|e| StorageIOError::write(&e))?;
        self.put_meta("last_purged_log_id", &serialized)
            .map_err(|e| StorageIOError::write(&e))?;

        // Remove all entries up to and including log_id.index()
        let mut batch = sled::Batch::default();
        for result in tree.range(..=log_id.get_log_id().index.to_be_bytes()) {
            let (key, _) = result.map_err(|e| StorageIOError::write(&e))?;
            batch.remove(key);
        }

        tree.apply_batch(batch)
            .map_err(|e| StorageIOError::write(&e))?;
        Ok(())
    }
}
