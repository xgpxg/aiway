use crate::{LoadBalance, Node, SelectOptions, healthy_indices};
use std::sync::atomic::{AtomicUsize, Ordering};

/// 轮询负载均衡
pub struct RoundRobinLoadBalance {
    state: AtomicUsize,
}

impl RoundRobinLoadBalance {
    pub fn new() -> Self {
        Self {
            state: AtomicUsize::new(0),
        }
    }
}

impl Default for RoundRobinLoadBalance {
    fn default() -> Self {
        Self::new()
    }
}

impl LoadBalance for RoundRobinLoadBalance {
    fn order(&self, nodes: &[Node], unhealthy: &[usize], _opt: &SelectOptions) -> Vec<usize> {
        // 剔除不健康实例
        let mut healthy = healthy_indices(nodes.len(), unhealthy);
        if healthy.is_empty() {
            return healthy;
        }
        // 推进计数器一次，从当前轮询位置开始旋转
        let counter = self.state.fetch_add(1, Ordering::Relaxed);
        let start = counter % healthy.len();
        healthy.rotate_left(start);
        healthy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotates_primary() {
        let nodes = [
            Node { addr: "a", weight: 1 },
            Node { addr: "b", weight: 1 },
            Node { addr: "c", weight: 1 },
        ];
        let lb = RoundRobinLoadBalance::new();
        let opt = SelectOptions::default();
        let p0 = lb.order(&nodes, &[], &opt)[0];
        let p1 = lb.order(&nodes, &[], &opt)[0];
        let p2 = lb.order(&nodes, &[], &opt)[0];
        let p3 = lb.order(&nodes, &[], &opt)[0];
        // 三次不同，第四次回到起点
        assert_ne!(p0, p1);
        assert_ne!(p1, p2);
        assert_ne!(p0, p2);
        assert_eq!(p0, p3);
    }

    #[test]
    fn skips_unhealthy() {
        let nodes = [
            Node { addr: "a", weight: 1 },
            Node { addr: "b", weight: 1 },
        ];
        let lb = RoundRobinLoadBalance::new();
        let order = lb.order(&nodes, &[1], &SelectOptions::default());
        assert_eq!(order, vec![0]);
    }
}
