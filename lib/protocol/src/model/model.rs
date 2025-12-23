use crate::model::provider::Provider;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    /// 模型名称，全局唯一
    pub name: String,
    /// 模型提供商
    pub providers: Vec<Provider>,
    /// 负载策略
    pub lb: LbStrategy,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum LbStrategy {
    /// 随机
    #[default]
    Random,
    /// 轮询
    RoundRobin,
    /// 权重随机
    ///
    /// 按提供商配置的权重随机
    WeightedRandom,
}
