use crate::{LoadBalance, Node, SelectOptions, healthy_indices};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;

/// 每个真实节点的基础虚拟节点数
const BASE_VNODES: usize = 128;
/// 单节点虚拟节点上限，防止超大权重导致环膨胀
const MAX_VNODES_PER_NODE: usize = 2048;

/// 哈希环负载均衡
///
/// IP 哈希（按客户端真实 IP）基于此哈希环实现，
/// 哈希键由调用方通过 [`SelectOptions::key`] 传入。
///
/// - 相同 key 稳定映射到同一节点（会话保持）
/// - 节点上下线时，仅影响环上相邻的一小段 key
/// - 权重越高的节点占据越多虚拟节点，承接越多流量
pub struct HashLoadBalance {
    /// 缓存的哈希环，节点集合（地址+权重）变化时重建
    cache: Mutex<Option<RingCache>>,
}

struct RingCache {
    signature: u64,
    /// 按虚拟节点哈希升序排列的 (hash, 真实节点索引)
    ring: Vec<(u64, usize)>,
}

impl HashLoadBalance {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(None),
        }
    }
}

impl Default for HashLoadBalance {
    fn default() -> Self {
        Self::new()
    }
}

fn hash64(s: &str) -> u64 {
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

impl LoadBalance for HashLoadBalance {
    fn order(&self, nodes: &[Node], unhealthy: &[usize], opt: &SelectOptions) -> Vec<usize> {
        let healthy = healthy_indices(nodes.len(), unhealthy);
        if healthy.is_empty() {
            return healthy;
        }

        // 无哈希键时退化为随机
        let Some(key) = opt.key else {
            let mut h = healthy.clone();
            fastrand::shuffle(&mut h);
            return h;
        };

        let mut cache = self.cache.lock().unwrap();
        let signature = ring_signature(nodes);
        let need_rebuild = match cache.as_ref() {
            Some(c) => c.signature != signature,
            None => true,
        };
        if need_rebuild {
            *cache = Some(RingCache {
                signature,
                ring: build_ring(nodes),
            });
        }
        let ring = &cache.as_ref().unwrap().ring;

        // 定位 key 在环上的起点
        let point = hash64(key);
        let mut start = ring.partition_point(|(h, _)| *h < point);
        if start >= ring.len() {
            start = 0;
        }

        // 从起点沿环遍历，按健康标记收集去重后的节点
        let mut seen = vec![false; nodes.len()];
        let mut order = Vec::with_capacity(healthy.len());
        for step in 0..ring.len() {
            let idx = ring[(start + step) % ring.len()].1;
            if unhealthy.contains(&idx) || seen[idx] {
                continue;
            }
            seen[idx] = true;
            order.push(idx);
            if order.len() == healthy.len() {
                break;
            }
        }

        // 兜底：环上未覆盖到的健康节点（正常不会发生）
        if order.len() < healthy.len() {
            for &i in &healthy {
                if !seen[i] {
                    order.push(i);
                }
            }
        }
        order
    }
}

/// 由节点地址与权重计算集合签名，用于判断环是否需要重建
fn ring_signature(nodes: &[Node]) -> u64 {
    let mut h = DefaultHasher::new();
    for n in nodes {
        n.addr.hash(&mut h);
        n.weight.hash(&mut h);
    }
    h.finish()
}

/// 构建哈希环：按权重分配虚拟节点
fn build_ring(nodes: &[Node]) -> Vec<(u64, usize)> {
    let mut ring = Vec::new();
    for (i, n) in nodes.iter().enumerate() {
        let vnodes = (n.weight.max(1) as usize)
            .saturating_mul(BASE_VNODES)
            .min(MAX_VNODES_PER_NODE);
        for v in 0..vnodes {
            let h = hash64(&format!("{}#{}", n.addr, v));
            ring.push((h, i));
        }
    }
    ring.sort_by_key(|(h, _)| *h);
    ring
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sticky_for_same_key() {
        let nodes = [
            Node { addr: "a", weight: 1 },
            Node { addr: "b", weight: 1 },
            Node { addr: "c", weight: 1 },
        ];
        let lb = HashLoadBalance::new();
        let opt = SelectOptions {
            key: Some("/api/user/123"),
            conn_counts: None,
        };
        let first = lb.order(&nodes, &[], &opt)[0];
        for _ in 0..50 {
            assert_eq!(lb.order(&nodes, &[], &opt)[0], first);
        }
    }

    #[test]
    fn skips_unhealthy_but_sticky() {
        let nodes = [
            Node { addr: "a", weight: 1 },
            Node { addr: "b", weight: 1 },
            Node { addr: "c", weight: 1 },
        ];
        let lb = HashLoadBalance::new();
        let opt = SelectOptions {
            key: Some("1.2.3.4"),
            conn_counts: None,
        };
        let all = lb.order(&nodes, &[], &opt);
        assert_eq!(all.len(), 3);
        // 剔除首个健康节点后，仍应返回其余全部且不含被剔除者
        let removed = all[0];
        let rest = lb.order(&nodes, &[removed], &opt);
        assert!(!rest.contains(&removed));
        assert_eq!(rest.len(), 2);
    }

    #[test]
    fn different_keys_spread() {
        let nodes = [
            Node { addr: "a", weight: 1 },
            Node { addr: "b", weight: 1 },
            Node { addr: "c", weight: 1 },
        ];
        let lb = HashLoadBalance::new();
        let mut hit = [0usize; 3];
        for i in 0..300 {
            let opt = SelectOptions {
                key: Some(&format!("/path/{}", i)),
                conn_counts: None,
            };
            hit[lb.order(&nodes, &[], &opt)[0]] += 1;
        }
        // 三个节点都应被命中
        assert!(hit.iter().all(|&c| c > 0), "hit={:?}", hit);
    }

    #[test]
    fn no_key_falls_back() {
        let nodes = [Node { addr: "a", weight: 1 }, Node { addr: "b", weight: 1 }];
        let lb = HashLoadBalance::new();
        let order = lb.order(&nodes, &[], &SelectOptions::default());
        assert_eq!(order.len(), 2);
    }
}
