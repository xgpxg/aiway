use crate::{LoadBalance, Node, SelectOptions, healthy_indices};

/// 随机负载均衡
pub struct RandomLoadBalance {}

impl RandomLoadBalance {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for RandomLoadBalance {
    fn default() -> Self {
        Self::new()
    }
}

impl LoadBalance for RandomLoadBalance {
    fn order(&self, nodes: &[Node], unhealthy: &[usize], _opt: &SelectOptions) -> Vec<usize> {
        // 剔除不健康实例后随机打乱
        let mut healthy = healthy_indices(nodes.len(), unhealthy);
        fastrand::shuffle(&mut healthy);
        healthy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_nodes() {
        let lb = RandomLoadBalance::new();
        assert!(lb.order(&[], &[], &SelectOptions::default()).is_empty());
    }

    #[test]
    fn covers_all_healthy() {
        let nodes = [
            Node { addr: "a", weight: 1 },
            Node { addr: "b", weight: 1 },
            Node { addr: "c", weight: 1 },
        ];
        let lb = RandomLoadBalance::new();
        let mut order = lb.order(&nodes, &[1], &SelectOptions::default());
        order.sort();
        // 索引 1 不健康，应被剔除
        assert_eq!(order, vec![0, 2]);
    }
}
