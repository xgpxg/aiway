use crate::{LoadBalance, Node, SelectOptions, healthy_indices};

/// 加权随机负载均衡
///
/// 按健康节点权重之和取随机落点，权重越中被命中概率越大。
pub struct WeightedRandomLoadBalance {}

impl WeightedRandomLoadBalance {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for WeightedRandomLoadBalance {
    fn default() -> Self {
        Self::new()
    }
}

impl LoadBalance for WeightedRandomLoadBalance {
    fn order(&self, nodes: &[Node], unhealthy: &[usize], _opt: &SelectOptions) -> Vec<usize> {
        let mut healthy = healthy_indices(nodes.len(), unhealthy);
        if healthy.is_empty() {
            return healthy;
        }

        let total: u64 = healthy.iter().map(|&i| nodes[i].weight as u64).sum();
        // 落点 [0, total)
        let mut point = fastrand::u64(0..total);
        let mut picked = healthy[healthy.len() - 1];
        for &i in &healthy {
            if point < nodes[i].weight as u64 {
                picked = i;
                break;
            }
            point -= nodes[i].weight as u64;
        }

        // 主实例置顶，其余健康节点随机打乱作为候选
        healthy.retain(|&i| i != picked);
        fastrand::shuffle(&mut healthy);
        let mut order = Vec::with_capacity(healthy.len() + 1);
        order.push(picked);
        order.extend(healthy);
        order
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roughly_follows_weight() {
        let nodes = [
            Node { addr: "a", weight: 9 },
            Node { addr: "b", weight: 1 },
        ];
        let lb = WeightedRandomLoadBalance::new();
        let opt = SelectOptions::default();
        let mut a = 0;
        let n = 20000;
        for _ in 0..n {
            if lb.order(&nodes, &[], &opt)[0] == 0 {
                a += 1;
            }
        }
        let ratio = a as f64 / n as f64;
        // 期望约 0.9，留容差
        assert!((ratio - 0.9).abs() < 0.05, "ratio={}", ratio);
    }
}
