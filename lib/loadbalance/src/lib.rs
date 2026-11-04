//! # 网关服务端的负载均衡

mod hash_ring;
mod least_conn;
mod random;
mod round;
mod weighted_random;
mod weighted_round;

pub use hash_ring::HashLoadBalance;
pub use least_conn::LeastConnLoadBalance;
pub use random::RandomLoadBalance;
pub use round::RoundRobinLoadBalance;
pub use weighted_random::WeightedRandomLoadBalance;
pub use weighted_round::WeightedRoundRobinLoadBalance;

/// 负载均衡输入节点视图
///
/// 借用服务节点的地址与权重，避免在热路径上产生克隆。
#[derive(Debug, Clone, Copy)]
pub struct Node<'a> {
    /// 节点地址
    pub addr: &'a str,
    /// 权重，最小为 1
    pub weight: u32,
}

/// 选择时的附加输入
///
/// 不同策略按需取用：哈希类策略用 `key`，最少连接用 `conn_counts`。
#[derive(Debug, Clone, Copy, Default)]
pub struct SelectOptions<'a> {
    /// 哈希键：IP 哈希传客户端真实 IP
    pub key: Option<&'a str>,
    /// 与 `nodes` 一一对齐的实时连接数，供最少连接策略使用
    pub conn_counts: Option<&'a [u32]>,
}

/// 负载均衡器 trait
pub trait LoadBalance: Send + Sync {
    /// 单点选择一个实例，返回其在 `nodes` 中的索引。
    ///
    /// 默认实现复用 [`LoadBalance::order`]（不健康列表为空），取排序后的首个。
    fn select(&self, nodes: &[Node], opt: &SelectOptions) -> Option<usize> {
        self.order(nodes, &[], opt).into_iter().next()
    }

    /// 返回按策略排序后的健康实例索引，第 0 个为主实例，其余为故障切换候选。
    ///
    /// - `unhealthy`：需要剔除的不健康实例索引
    fn order(&self, nodes: &[Node], unhealthy: &[usize], opt: &SelectOptions) -> Vec<usize>;
}

/// 计算健康实例索引（剔除 `unhealthy`），保持原有顺序。
pub(crate) fn healthy_indices(len: usize, unhealthy: &[usize]) -> Vec<usize> {
    (0..len).filter(|i| !unhealthy.contains(i)).collect()
}

/// 负载均衡错误类型
#[derive(Debug)]
pub enum LoadBalanceError {
    /// 获取服务实例列表失败
    GetInstancesError(String),
    /// 无可用实例
    NoAvailableInstance,
}

impl std::fmt::Display for LoadBalanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadBalanceError::GetInstancesError(e) => write!(f, "Failed to get instances: {}", e),
            LoadBalanceError::NoAvailableInstance => write!(f, "No available instance"),
        }
    }
}

impl std::error::Error for LoadBalanceError {}
