//! # 服务
//! 负责从配置中心加载服务并缓存。
//!
//! 实现流程：
//! - 初始化时，从配置中心获取配置key为`services`的配置项
//! - 反序列化为[`Vec<Service>`]
//! - 缓存服务
//! - 监听配置`routes.yaml`变更，重写获取服务并缓存
//!
//! TODO 以上内容需重写
//!
//! # 服务
//! 负责从控制台加载服务配置并缓存。
//!
//! 实现流程：
//! - 初始化时，尝试从控制台的`GET /api/v1/gateway/services`端点获取服务列表。
//! - 如果控制台无法连接，则退出，禁止启动。
//! - 反序列化响应结果为[`Vec<Service>`]
//! - 缓存服务列表到内存以及本地。
//! - 启动定时任务，每5秒从控制台拉取服务列表，校验hash值，如果不一致则更新本地服务列表。
//!
//! 服务定义：[`Service`]
//!
use crate::constants;
use conreg_client::AppConfig;
use dashmap::DashMap;
use loadbalance::LoadBalance;
use protocol::gateway;
use protocol::gateway::Route;
use protocol::gateway::service::LbStrategy;
use std::sync::{Arc, LazyLock};

pub struct Service {
    /// 路由表
    services: DashMap<String, Arc<LbService>>,
}

pub static SERVICES: LazyLock<Service> = LazyLock::new(Service::load);

impl Service {
    pub fn load() -> Self {
        let list = Self::fetch_services();

        let mut services = DashMap::new();
        for service in list.into_iter() {
            let lb_strategy = service.lb.clone();
            services.insert(
                service.id.clone(),
                Arc::new(LbService::new(service, lb_strategy)),
            );
        }

        Self::watch();

        Self { services }
    }

    fn fetch_services() -> Vec<gateway::Service> {
        // 从配置中心拿路由表
        let services = AppConfig::get::<Vec<gateway::Service>>("services").unwrap_or_default();
        log::info!("fetched {} services", services.len());
        log::debug!("services: {:?}", services);

        services
    }

    fn watch() {
        AppConfig::add_listener(constants::SERVICES_CONFIG_ID, |_| {
            let list = Self::fetch_services();

            SERVICES
                .services
                .retain(|_, item| list.iter().any(|s| s.id == item.service.id));

            for service in list.into_iter() {
                let lb_strategy = service.lb.clone();
                SERVICES.services.insert(
                    service.id.clone(),
                    Arc::new(LbService::new(service, lb_strategy)),
                );
            }
        });
    }

    pub fn get_instance(&self, service_id: &str) -> Option<String> {
        let service = SERVICES.services.get(service_id);
        if let Some(service) = service {
            return service.lb.select(&service.service.nodes);
        }
        None
    }
}

struct LbService {
    service: gateway::Service,
    lb: Box<dyn LoadBalance<String>>,
}

impl LbService {
    pub fn new(service: gateway::Service, strategy: LbStrategy) -> Self {
        let lb: Box<dyn LoadBalance<String>> = match strategy {
            LbStrategy::Random => Box::new(loadbalance::RandomLoadBalance::new()),
            LbStrategy::RoundRobin => Box::new(loadbalance::RoundRobinLoadBalance::new()),
        };
        Self { service, lb }
    }
}
