use crate::LoadBalance;
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

impl<T: Clone> LoadBalance<T> for RoundRobinLoadBalance {
    fn select(&self, instances: &[T]) -> Option<T> {
        if instances.is_empty() {
            return None;
        }

        if instances.len() == 1 {
            return Some(instances[0].clone());
        }

        let index = self.state.fetch_add(1, Ordering::Relaxed);
        let index = index % instances.len();

        Some(instances[index].clone())
    }

    fn select_all(&self, instances: &[T], unhealthy_indices: &[usize]) -> Vec<T> {
        if instances.is_empty() {
            return vec![];
        }

        // 推进计数器一次
        let counter = self.state.fetch_add(1, Ordering::Relaxed);

        // 过滤不健康实例，从轮询位置开始旋转
        let mut filtered: Vec<T> = instances
            .iter()
            .enumerate()
            .filter(|(i, _)| !unhealthy_indices.contains(i))
            .map(|(_, v)| v.clone())
            .collect();
        let flen = filtered.len();
        if flen > 0 {
            let start = counter % flen;
            filtered.rotate_left(start);
        }
        filtered
    }
}
