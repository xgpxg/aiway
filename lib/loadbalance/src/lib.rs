//! # 网关服务端的负载均衡
//!
//!
mod random;
mod round;

pub use random::RandomLoadBalance;
pub use round::RoundRobinLoadBalance;

/// 实例提供者
pub trait Instances<T: Clone> {
    /// 获取所有实例
    fn instances(&self) -> Vec<T>;
}

impl Instances<String> for Vec<String> {
    fn instances(&self) -> Vec<String> {
        self.clone()
    }
}

/// 负载均衡器 trait
pub trait LoadBalance<T: Clone>: Sync + Send {
    /// 从实例中选择一个
    fn select(&self, instances: &[T]) -> Option<T>;
}

/// 负载均衡错误类型
#[derive(Debug)]
pub enum LoadBalanceError {
    /// 获取服务实例列表失败
    GetInstancesError(String),
    /// 无可用实例
    NoAvailableInstance,
}

impl std::fmt::Display for LoadBalanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadBalanceError::GetInstancesError(e) => write!(f, "Failed to get instances: {}", e),
            LoadBalanceError::NoAvailableInstance => {
                write!(f, "No available instance")
            }
        }
    }
}

impl std::error::Error for LoadBalanceError {}

/// 负载均衡策略
#[derive(Debug, Default)]
pub enum LoadBalanceStrategy {
    /// 随机
    #[default]
    Random,
    /// 轮询
    RoundRobin,
}

impl LoadBalanceStrategy {
    #[allow(unused)]
    pub fn as_schema(&self) -> &str {
        match self {
            LoadBalanceStrategy::Random => "r",
            LoadBalanceStrategy::RoundRobin => "rr",
        }
    }
}
/*
/// 服务定义
pub struct Service<T> {
    /// 服务实例列表
    instances: Vec<T>,
    /// 负载均衡策略
    strategy: LoadBalanceStrategy,
}

impl<T: Clone> Service<T> {
    /// 创建新的服务
    pub fn new(instances: Vec<T>, strategy: LoadBalanceStrategy) -> Self {
        Self {
            instances,
            strategy,
        }
    }

    /// 获取服务实例
    pub fn instances(&self) -> &[T] {
        &self.instances
    }

    /// 获取负载均衡策略
    pub fn strategy(&self) -> &LoadBalanceStrategy {
        &self.strategy
    }
}

/// 负载均衡工厂
pub struct LoadBalanceFactory<T: Clone> {
    services: DashMap<String, (Box<dyn LoadBalance<T>>, Box<dyn Instances<T>>)>,
    random_lb: RandomLoadBalance,
    round_lb: RoundRobinLoadBalance,
}

impl<T: Clone> LoadBalanceFactory<T> {
    pub fn new() -> Self {
        Self {
            services: DashMap::new(),
            random_lb: RandomLoadBalance::new(),
            round_lb: RoundRobinLoadBalance::new(),
        }
    }

    pub fn add(&self, key: &str, service: Box<dyn Instances<T>>, strategy: LoadBalanceStrategy) {
        let strategy = match strategy {
            LoadBalanceStrategy::Random => {
                Box::new(RandomLoadBalance::new()) as Box<dyn LoadBalance<T>>
            }
            LoadBalanceStrategy::RoundRobin => {
                Box::new(RoundRobinLoadBalance::new()) as Box<dyn LoadBalance<T>>
            }
        };
        self.services.insert(key.to_string(), (strategy, service));
    }

    pub fn get_instance(&self, service_id: &str) -> Result<T, LoadBalanceError> {
        let service = self
            .services
            .get(service_id)
            .ok_or(LoadBalanceError::NoAvailableInstance)?;

        let instances = service.1.instances();
        if instances.is_empty() {
            return Err(LoadBalanceError::NoAvailableInstance);
        }

        let lb = &service.0;
        lb.select(&instances)
            .ok_or(LoadBalanceError::NoAvailableInstance)
    }
}
*/
