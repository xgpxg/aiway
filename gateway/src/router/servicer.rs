//! # 服务
//! 负责从配置中心加载服务并缓存。
//!
//! 实现流程：
//! - 初始化时，从配置中心获取配置key为`services`的配置项
//! - 反序列化为[`Vec<Servicer>`]
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
//! - 反序列化响应结果为[`Vec<Servicer>`]
//! - 缓存服务列表到内存以及本地。
//! - 启动定时任务，每5秒从控制台拉取服务列表，校验hash值，如果不一致则更新本地服务列表。
//!
//! 服务定义：[`Servicer`]
//!

use crate::router::client::INNER_HTTP_CLIENT;
use dashmap::DashMap;
use loadbalance::LoadBalance;
use protocol::gateway;
use protocol::gateway::service::LbStrategy;
use std::collections::BTreeMap;
use std::process::exit;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

pub struct Servicer {
    /// 服务表
    services: DashMap<String, Arc<LbService>>,
}

pub static SERVICES: OnceLock<Servicer> = OnceLock::new();

impl Servicer {
    pub async fn init() {
        if let Err(e) = Self::load().await {
            log::error!("{}", e);
            exit(1)
        }
    }
    pub async fn load() -> anyhow::Result<()> {
        let list = Self::fetch_services().await?;
        log::info!("loaded {} services", list.len());

        let services = DashMap::new();
        for service in list.into_iter() {
            let lb_strategy = service.lb.clone();
            services.insert(
                service.name.clone(),
                Arc::new(LbService::new(service, lb_strategy)),
            );
        }

        SERVICES.get_or_init(|| Self { services });

        Self::watch();

        Ok(())
    }

    async fn fetch_services() -> anyhow::Result<Vec<gateway::Service>> {
        INNER_HTTP_CLIENT.fetch_services().await
    }

    const INTERVAL: Duration = Duration::from_secs(5);
    fn watch() {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Self::INTERVAL);
            loop {
                interval.tick().await;

                let list = Self::fetch_services().await;

                let list = match list {
                    Ok(list) => list,
                    Err(e) => {
                        log::error!("{}", e);
                        continue;
                    }
                };

                let old_services = {
                    SERVICES
                        .get()
                        .unwrap()
                        .services
                        .iter()
                        .map(|item| (item.key().clone(), item.value().service.clone()))
                        .collect::<BTreeMap<String, gateway::Service>>()
                };
                let new_services = {
                    list.iter()
                        .map(|item| (item.name.clone(), item.clone()))
                        .collect::<BTreeMap<String, gateway::Service>>()
                };

                let old = serde_json::to_string(&old_services).unwrap();
                let new = serde_json::to_string(&new_services).unwrap();

                if old == new {
                    log::debug!("services not changed");
                    continue;
                }

                SERVICES
                    .get()
                    .unwrap()
                    .services
                    .retain(|_, item| list.iter().any(|s| s.name == item.service.name));

                for service in list.into_iter() {
                    let lb_strategy = service.lb.clone();
                    SERVICES.get().unwrap().services.insert(
                        service.name.clone(),
                        Arc::new(LbService::new(service, lb_strategy)),
                    );
                }
            }
        });
    }

    pub fn get_instance(&self, service_id: &str) -> Option<String> {
        let service = SERVICES.get().unwrap().services.get(service_id);
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
