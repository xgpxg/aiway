use crate::LoadBalance;

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

impl<T: Clone> LoadBalance<T> for RandomLoadBalance {
    fn select(&self, instances: &[T]) -> Option<T> {
        if instances.is_empty() {
            return None;
        }

        if instances.len() == 1 {
            return Some(instances[0].clone());
        }

        let index = fastrand::usize(0..instances.len());
        Some(instances[index].clone())
    }

    fn select_all(&self, instances: &[T], unhealthy_indices: &[usize]) -> Vec<T> {
        if instances.is_empty() {
            return vec![];
        }

        // 过滤不健康实例，健康实例随机打乱
        let mut healthy: Vec<T> = instances
            .iter()
            .enumerate()
            .filter(|(i, _)| !unhealthy_indices.contains(i))
            .map(|(_, v)| v.clone())
            .collect();
        fastrand::shuffle(&mut healthy);
        healthy
    }
}
