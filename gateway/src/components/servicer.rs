//! # 服务
//! 负责从控制台加载服务配置并缓存。
//!
//! 实现流程：
//! - 初始化时，尝试从控制台的`GET /api/v1/gateway/services`端点获取服务列表。
//! - 如果控制台无法连接，则退出，禁止启动。
//! - 反序列化响应结果为[`Vec<Servicer>`]
//! - 缓存服务列表到内存以及本地。
//! - 启动定时任务，每5秒从控制台拉取服务列表，校验hash值，如果不一致则更新本地服务列表。
//!
//! 服务定义：[`Servicer`]
//!

use crate::components::client::INNER_HTTP_CLIENT;
use aiway_protocol::common::constants::GATEWAY_LOCAL_SOCK_PATH;
use aiway_protocol::gateway;
use aiway_protocol::gateway::service::{LbStrategy, ServiceNode};
use dashmap::DashMap;
use loadbalance::{
    HashLoadBalance, LeastConnLoadBalance, LoadBalance, Node, RandomLoadBalance,
    RoundRobinLoadBalance, SelectOptions, WeightedRandomLoadBalance,
    WeightedRoundRobinLoadBalance,
};
use std::process::exit;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, LazyLock, OnceLock};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// 服务管理器
pub static SERVICES: OnceLock<Servicer> = OnceLock::new();

/// 不健康实例标记，key = 实例地址，value = 标记时间
static UNHEALTHY_INSTANCES: LazyLock<DashMap<String, Instant>> = LazyLock::new(DashMap::new);

/// 不健康标记的 TTL（秒），过期后自动恢复尝试
const UNHEALTHY_TTL: Duration = Duration::from_secs(30);

/// 每个实例地址的在途连接数，供最少连接（LeastConn）策略使用
static CONNS: LazyLock<DashMap<String, AtomicU32>> = LazyLock::new(DashMap::new);

/// 本地服务，用于处理模型和MCP调用
pub static LOCAL_SERVICE: LazyLock<gateway::Service> = LazyLock::new(|| gateway::Service {
    name: "__local__".to_string(),
    nodes: vec![ServiceNode::new(GATEWAY_LOCAL_SOCK_PATH)],
    lb: LbStrategy::Random,
});

pub struct Servicer {
    services: DashMap<String, Arc<LbService>>,
    hash: Arc<RwLock<String>>,
}

impl Servicer {
    pub async fn init() {
        if let Err(e) = Self::load().await {
            log::error!("{}", e);
            exit(1)
        }
    }
    pub async fn load() -> anyhow::Result<()> {
        let list = Self::fetch_services().await?;
        log::info!("loaded {} services", list.len());

        let hash = md5::compute(serde_json::to_string(&list)?);
        let hash = format!("{:x}", hash);

        SERVICES.get_or_init(|| Self {
            services: Self::process_services(list),
            hash: Arc::new(RwLock::new(hash)),
        });

        Self::watch();

        Ok(())
    }

    fn process_services(list: Vec<gateway::Service>) -> DashMap<String, Arc<LbService>> {
        let services = DashMap::new();
        for service in list.into_iter() {
            let lb_strategy = service.lb.clone();
            services.insert(
                service.name.clone(),
                Arc::new(LbService::new(service, lb_strategy)),
            );
        }
        services
    }

    async fn fetch_services() -> anyhow::Result<Vec<gateway::Service>> {
        let mut services = INNER_HTTP_CLIENT.fetch_services().await?;
        services.push(LOCAL_SERVICE.clone());
        Ok(services)
    }

    const INTERVAL: Duration = Duration::from_secs(5);
    fn watch() {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Self::INTERVAL);
            loop {
                interval.tick().await;

                let list = Self::fetch_services().await;

                let list = match list {
                    Ok(list) => list,
                    Err(e) => {
                        log::error!("{}", e);
                        continue;
                    }
                };

                let hash = md5::compute(serde_json::to_string(&list).unwrap());
                let hash = format!("{:x}", hash);

                let old_services = SERVICES.get().unwrap();
                if hash == *old_services.hash.read().await {
                    log::debug!("services not changed, wait next interval");
                    continue;
                }

                log::info!("loaded {} services", list.len());

                let new_services = Self::process_services(list);
                {
                    old_services
                        .services
                        .retain(|name, _| new_services.contains_key(name));

                    new_services.into_iter().for_each(|(name, service)| {
                        old_services.services.insert(name, service.clone());
                    });
                    *old_services.hash.write().await = hash;
                }
            }
        });
    }

    /// 获取一个服务实例
    ///
    /// 这个暂且保留，MCP处理时用到，MCP目前用到的是默认的负载均衡策略（随机）
    /// 未来可能会移除
    pub fn get_instance(service_id: &str) -> Option<String> {
        let servicer = SERVICES.get()?;
        let service = servicer.services.get(service_id).map(|s| Arc::clone(&s))?;
        let nodes = &service.service.nodes;
        let views: Vec<Node> = nodes
            .iter()
            .map(|n| Node {
                addr: &n.addr,
                weight: n.weight,
            })
            .collect();

        let idx = service.lb.select(&views, &SelectOptions::default())?;
        Some(nodes[idx].addr.clone())
    }

    /// 返回按负载策略排序的所有健康实例地址，第 0 个为主实例，其余为故障切换候选。
    /// 用于负载均衡的选择与故障自动切换。
    ///
    /// - `client_ip`：IP 哈希（IpHash）的哈希键
    pub fn get_instances(service_id: &str, client_ip: Option<&str>) -> Vec<String> {
        //SAFE: SERVICES已初始化
        let Some(servicer) = SERVICES.get() else {
            return vec![];
        };
        // 克隆 Arc 后立即释放锁，避免持锁期间执行耗时操作
        let service = match servicer.services.get(service_id) {
            Some(s) => Arc::clone(&s),
            None => return vec![],
        };

        // 所有实例
        let nodes = &service.service.nodes;
        if nodes.is_empty() {
            return vec![];
        }
        let views: Vec<Node> = nodes
            .iter()
            .map(|n| Node {
                addr: &n.addr,
                weight: n.weight,
            })
            .collect();
        let now = Instant::now();

        // 清理过期的不健康标记，并收集仍有效的不健康实例索引
        // 先读后写分离，避免 DashMap 读写锁死锁
        let unhealthy_indices: Vec<usize> = nodes
            .iter()
            .enumerate()
            .filter_map(|(i, node)| {
                // 先读：获取标记状态（读锁立即释放）
                let is_unhealthy = UNHEALTHY_INSTANCES
                    .get(&node.addr)
                    .map(|marked_at| now.duration_since(*marked_at) < UNHEALTHY_TTL)
                    .unwrap_or(false);
                if is_unhealthy {
                    Some(i)
                } else {
                    // 过期或不存在，移除标记
                    UNHEALTHY_INSTANCES.remove(&node.addr);
                    None
                }
            })
            .collect();

        // 实时连接数快照，供 LeastConn 使用
        let conn_counts: Vec<u32> = nodes.iter().map(|n| Self::conn(&n.addr)).collect();

        // 按策略选择哈希键
        let key: Option<&str> = match service.service.lb {
            LbStrategy::IpHash => client_ip,
            _ => None,
        };

        let opt = SelectOptions {
            key,
            conn_counts: Some(&conn_counts),
        };
        service
            .lb
            .order(&views, &unhealthy_indices, &opt)
            .into_iter()
            .map(|i| nodes[i].addr.clone())
            .collect()
    }

    /// 将实例标记为不健康，后续请求会优先选择其他实例
    pub fn mark_unhealthy(instance: &str) {
        UNHEALTHY_INSTANCES.insert(instance.to_string(), Instant::now());
        log::warn!("mark instance unhealthy: {}", instance);
    }

    /// 实例在途连接数 +1
    pub fn inc_conn(addr: &str) {
        CONNS
            .entry(addr.to_string())
            .or_default()
            .fetch_add(1, Ordering::Relaxed);
    }

    /// 实例在途连接数 -1（不会小于 0）
    pub fn dec_conn(addr: &str) {
        if let Some(c) = CONNS.get(addr) {
            if c.load(Ordering::Relaxed) > 0 {
                c.fetch_sub(1, Ordering::Relaxed);
            }
        }
    }

    /// 读取实例当前在途连接数
    fn conn(addr: &str) -> u32 {
        CONNS
            .get(addr)
             .map(|c| c.load(Ordering::Relaxed))
            .unwrap_or(0)
    }
}

struct LbService {
    service: gateway::Service,
    lb: Box<dyn LoadBalance>,
}

impl LbService {
    pub fn new(service: gateway::Service, strategy: LbStrategy) -> Self {
        let lb: Box<dyn LoadBalance> = match strategy {
            LbStrategy::Random => Box::new(RandomLoadBalance::new()),
            LbStrategy::RoundRobin => Box::new(RoundRobinLoadBalance::new()),
            LbStrategy::WeightedRoundRobin => Box::new(WeightedRoundRobinLoadBalance::new()),
            LbStrategy::WeightedRandom => Box::new(WeightedRandomLoadBalance::new()),
            LbStrategy::LeastConn => Box::new(LeastConnLoadBalance::new()),
            // IP 哈希基于哈希环实现
            LbStrategy::IpHash => Box::new(HashLoadBalance::new()),
        };
        Self { service, lb }
    }
}
