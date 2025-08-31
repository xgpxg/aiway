use crate::Args;
use crate::config::raft::store::StateMachineData;
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
mod store;

// 1. 定义客户端的请求和响应
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "cmd", content = "data")]
pub enum RaftRequest {
    Set { key: String, value: String },
    Delete { key: String },
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

/// 节点ID
pub type NodeId = u64;

pub struct App {
    /// 节点ID
    pub id: NodeId,
    /// 节点地址
    pub addr: String,
    /// Raft协议
    pub raft: Raft,
    /// 状态机
    /// 注意这个需要共享状态，Raft应用apply后会修改这个，在读取数据时，也从这里读
    pub state_machine: Arc<RwLock<StateMachineData>>,
    /// 应用额外数据
    #[allow(unused)]
    pub other: Arc<RwLock<HashMap<String, String>>>,
}

pub async fn new_raft_app(node_id: NodeId) -> App {
    let args = Args::parse();

    let config = Config {
        heartbeat_interval: 500,
        election_timeout_min: 1500,
        election_timeout_max: 3000,
        ..Default::default()
    };

    // 校验配置是否有效
    let config = Arc::new(config.validate().unwrap());

    // 创建日志存储和状态机存储
    let (log_store, state_machine_store): (LogStore, StateMachine) =
        store::new(args.data_dir).await;

    // 创建网络
    let network = Network {};

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

    let addr = format!("{}:{}", args.address, args.port);

    App {
        id: node_id,
        addr,
        raft,
        state_machine,
        other: Arc::new(Default::default()),
    }
}
