use crate::config::raft::store::StateMachineData;
use openraft::Config;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Cursor;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod api;
mod declare_types;
pub mod network;
mod store;

// 1. 定义客户端的请求和响应
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "cmd", content = "data")]
pub enum RaftRequest {
    Set { key: String, value: String },
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

/// 节点ID
pub type NodeId = u64;

pub struct App {
    /// 节点ID
    pub id: NodeId,
    /// 节点地址
    pub addr: String,
    /// Raft协议
    pub raft: Raft,
    /// 额外数据
    pub key_values: Arc<RwLock<BTreeMap<String, String>>>,
    /// 状态机
    pub state_machine: Arc<RwLock<StateMachineData>>,
}

pub async fn new_raft_app(node_id: NodeId) -> App {
    let config = Config {
        heartbeat_interval: 500,
        election_timeout_min: 1500,
        election_timeout_max: 3000,
        ..Default::default()
    };

    let config = Arc::new(config.validate().unwrap());

    let (log_store, state_machine_store): (LogStore, StateMachine) = store::new("./data").await;
    let network = network::NetworkFactory {};

    let state_machine = state_machine_store.data.clone();

    let raft = Raft::new(
        node_id,
        config.clone(),
        network,
        log_store.clone(),
        state_machine_store,
    )
    .await
    .unwrap();

    let addr = format!(
        "{}:{}",
        local_ip_address::local_ip().unwrap().to_string(),
        5000
    );

    App {
        id: node_id,
        addr,
        raft,
        key_values: Arc::new(Default::default()),
        state_machine,
    }
}
