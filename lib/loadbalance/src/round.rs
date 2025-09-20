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
}
