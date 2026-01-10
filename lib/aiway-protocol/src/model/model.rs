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
    /// 总权重，由控制台返回
    pub total_weight: u32,
    /// 轮询索引，实时计算，不参与eq计算
    #[serde(skip)]
    pub round_robin_index: u64,
}

impl PartialEq for Model {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.providers == other.providers
            && self.lb == other.lb
            && self.total_weight == other.total_weight
    }
}

impl Eq for Model {}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
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
