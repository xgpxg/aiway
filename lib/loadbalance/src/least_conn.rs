use crate::{LoadBalance, Node, SelectOptions, healthy_indices};

/// 最少连接负载均衡
///
/// 选择当前在途连接数最少的健康节点。连接数由调用方（网关）通过
/// [`SelectOptions::conn_counts`] 传入，需与 `nodes` 一一对齐。
pub struct LeastConnLoadBalance {}

impl LeastConnLoadBalance {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for LeastConnLoadBalance {
    fn default() -> Self {
        Self::new()
    }
}

impl LoadBalance for LeastConnLoadBalance {
    fn order(&self, nodes: &[Node], unhealthy: &[usize], opt: &SelectOptions) -> Vec<usize> {
        let mut healthy = healthy_indices(nodes.len(), unhealthy);
        if healthy.is_empty() {
            return healthy;
        }

        // 有连接数快照时按连接数升序，否则退化为随机
        match opt.conn_counts {
            Some(counts) if counts.len() == nodes.len() => {
                healthy.sort_by_key(|&i| counts[i]);
            }
            _ => {
                fastrand::shuffle(&mut healthy);
            }
        }
        healthy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_min_conn() {
        let nodes = [
            Node { addr: "a", weight: 1 },
            Node { addr: "b", weight: 1 },
            Node { addr: "c", weight: 1 },
        ];
        let counts = [10u32, 2, 7];
        let lb = LeastConnLoadBalance::new();
        let opt = SelectOptions {
            key: None,
            conn_counts: Some(&counts),
        };
        let order = lb.order(&nodes, &[], &opt);
        // 连接数升序：b(2) < c(7) < a(10)
        assert_eq!(order, vec![1, 2, 0]);
    }

    #[test]
    fn skips_unhealthy() {
        let nodes = [
            Node { addr: "a", weight: 1 },
            Node { addr: "b", weight: 1 },
        ];
        let counts = [0u32, 100];
        let lb = LeastConnLoadBalance::new();
        let opt = SelectOptions {
            key: None,
            conn_counts: Some(&counts),
        };
        // 索引 0 不健康，即使连接数最少也应被剔除
        let order = lb.order(&nodes, &[0], &opt);
        assert_eq!(order, vec![1]);
    }
}
