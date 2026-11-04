use crate::{LoadBalance, Node, SelectOptions, healthy_indices};
use std::sync::Mutex;

/// 加权轮询负载均衡（nginx 平滑加权轮询算法）
///
/// 每个节点维护一个当前权重 `current`，每轮：
/// 1. 所有健康节点 `current += weight`
/// 2. 选出 `current` 最大的节点作为主实例
/// 3. 该节点 `current -= total_weight`
///
/// 这样既能按权重比例分配，又能让高权重节点平滑地分散到各轮，而不会连续命中。
pub struct WeightedRoundRobinLoadBalance {
    /// 与节点索引对齐的当前权重，节点集合变化时重建
    state: Mutex<Vec<i64>>,
}

impl WeightedRoundRobinLoadBalance {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(Vec::new()),
        }
    }
}

impl Default for WeightedRoundRobinLoadBalance {
    fn default() -> Self {
        Self::new()
    }
}

impl LoadBalance for WeightedRoundRobinLoadBalance {
    fn order(&self, nodes: &[Node], unhealthy: &[usize], _opt: &SelectOptions) -> Vec<usize> {
        let healthy = healthy_indices(nodes.len(), unhealthy);
        if healthy.is_empty() {
            return healthy;
        }

        let mut current = self.state.lock().unwrap();
        // 节点集合变化时重建状态
        if current.len() != nodes.len() {
            *current = vec![0i64; nodes.len()];
        }

        // 健康节点权重总和
        let total: i64 = healthy.iter().map(|&i| nodes[i].weight as i64).sum();

        // 1. 累加
        for &i in &healthy {
            current[i] += nodes[i].weight as i64;
        }

        // 2. 选出 current 最大的健康节点
        let mut best = healthy[0];
        for &i in &healthy {
            if current[i] > current[best] {
                best = i;
            }
        }

        // 3. 扣减
        current[best] -= total;

        // 主实例置顶，其余健康节点保持原有顺序作为故障切换候选
        let mut order = Vec::with_capacity(healthy.len());
        order.push(best);
        order.extend(healthy.into_iter().filter(|&i| i != best));
        order
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distributes_by_weight() {
        let nodes = [
            Node { addr: "a", weight: 5 },
            Node { addr: "b", weight: 1 },
            Node { addr: "c", weight: 1 },
        ];
        let lb = WeightedRoundRobinLoadBalance::new();
        let opt = SelectOptions::default();

        let mut counts = [0; 3];
        for _ in 0..7 {
            let primary = lb.order(&nodes, &[], &opt)[0];
            counts[primary] += 1;
        }
        // 权重 5:1:1，共 7 次，应恰好命中 a 5 次、b/c 各 1 次
        assert_eq!(counts, [5, 1, 1]);
    }
}
