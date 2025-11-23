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
}
