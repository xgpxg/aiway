//! # 接入层代理 - L4 透传 + TLS 终止

use aiway_protocol::gateway::GatewayNodeInfo;
use std::sync::{Arc, RwLock};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

/// 节点池，维护当前可用的网关节点列表
///
/// 默认轮询
pub struct NodePool {
    /// 当前可用的节点列表
    nodes: RwLock<Vec<GatewayNodeInfo>>,
    /// 轮询计数器
    index: std::sync::atomic::AtomicUsize,
}

impl NodePool {
    pub fn new(nodes: Vec<GatewayNodeInfo>) -> Self {
        Self {
            nodes: RwLock::new(nodes),
            index: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// 获取下一个可用节点
    pub fn next(&self) -> Option<GatewayNodeInfo> {
        let guard = self.nodes.read().ok()?;
        if guard.is_empty() {
            return None;
        }
        let idx = self
            .index
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            % guard.len();
        Some(guard[idx].clone())
    }

    /// 更新节点列表
    pub fn update(&self, nodes: Vec<GatewayNodeInfo>) {
        if let Ok(mut guard) = self.nodes.write() {
            *guard = nodes;
        }
    }

    /// 获取当前节点数量
    pub fn len(&self) -> usize {
        self.nodes.read().map(|g| g.len()).unwrap_or(0)
    }
}

/// 接入点，L4 代理
///
/// 负责将请求转发到后端网关节点（纯 TCP 转发，不解析 HTTP）
pub struct AccessPoint {
    /// 节点池
    pub node_pool: Arc<NodePool>,
}

impl AccessPoint {
    pub fn new(node_pool: Arc<NodePool>) -> Self {
        Self { node_pool }
    }

    /// 处理单个连接：选择上游节点，建立连接，双向转发
    pub async fn handle_connection<S>(&self, downstream: S, peer_addr: std::net::SocketAddr)
    where
        S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
    {
        let node = match self.node_pool.next() {
            Some(n) => n,
            None => {
                log::error!("[{}] No available gateway nodes", peer_addr);
                return;
            }
        };

        let addr = node.addr();
        log::debug!("[{}] Forwarding to gateway: {}", peer_addr, addr);

        // 连接到上游网关
        let upstream = match TcpStream::connect(&addr).await {
            Ok(s) => s,
            Err(e) => {
                log::error!("[{}] Failed to connect upstream {}: {}", peer_addr, addr, e);
                return;
            }
        };

        let (mut upstream_reader, mut upstream_writer) = upstream.into_split();
        let (mut downstream_reader, mut downstream_writer) = tokio::io::split(downstream);

        // 双向转发
        let peer = peer_addr.to_string();
        let upstream_addr = addr.clone();

        let client_to_server = async {
            let result = tokio::io::copy(&mut downstream_reader, &mut upstream_writer).await;
            let _ = upstream_writer.shutdown().await;
            result
        };

        let server_to_client = async {
            let result = tokio::io::copy(&mut upstream_reader, &mut downstream_writer).await;
            let _ = downstream_writer.shutdown().await;
            result
        };

        tokio::select! {
            result = client_to_server => {
                match result {
                    Ok(bytes) => log::debug!("[{}] -> {} forwarded {} bytes", peer, upstream_addr, bytes),
                    Err(e) => log::debug!("[{}] -> {} error: {}", peer, upstream_addr, e),
                }
            }
            result = server_to_client => {
                match result {
                    Ok(bytes) => log::debug!("[{}] <- {} forwarded {} bytes", peer, upstream_addr, bytes),
                    Err(e) => log::debug!("[{}] <- {} error: {}", peer, upstream_addr, e),
                }
            }
        }
    }
}
