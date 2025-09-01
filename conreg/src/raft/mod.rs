use crate::Args;
use crate::raft::store::StateMachineData;
use crate::config::server::{ConfigApp, ConfigEntry};
use clap::Parser;
use openraft::Config;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod api;
mod declare_types;
pub mod network;
pub mod store;

// 1. 定义客户端的请求和响应
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "cmd", content = "data")]
pub enum RaftRequest {
    /// 设置键值对
    Set { key: String, value: String },
    /// 删除键值对
    Delete { key: String },
    /// 配置中心设置配置
    SetConfig { entry: ConfigEntry },
    /// 配置中心删除配置
    DeleteConfig { namespace_id: String, id: String },
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RaftResponse {
    pub value: Option<String>,
}

// 2. 定义Raft需要的类型配置
openraft::declare_raft_types!(
    pub TypeConfig:
        D = RaftRequest,
        R = RaftResponse,
);
pub type Raft = openraft::Raft<TypeConfig>;

// 3. 实现日志存储和状态机
pub type LogStore = store::SledLogStore<TypeConfig>;
pub type StateMachine = store::StateMachineStore;

// 4. 实现网络层
pub type Network = network::NetworkFactory;

pub trait Executor {
    fn write(key: impl Serialize, value: impl Serialize) -> anyhow::Result<()>;
    fn read(key: impl Serialize) -> anyhow::Result<Option<Vec<u8>>>;
}

/// 节点ID
pub type NodeId = u64;
