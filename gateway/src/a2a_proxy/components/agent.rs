use crate::a2a_proxy::components::client::AGENT_HTTP_CLIENT;
use aiway_protocol::a2a::Agent;
use dashmap::DashMap;
use std::collections::HashMap;
use std::process::exit;
use std::sync::OnceLock;
use std::time::Duration;

/// Agent 工厂
///
/// 管理所有已注册的 A2A Agent，定期从 Console 同步最新列表。
pub struct AgentFactory {
    agents: DashMap<String, Agent>,
}

pub static AGENT_FACTORY: OnceLock<AgentFactory> = OnceLock::new();

impl AgentFactory {
    pub async fn init() {
        if let Err(e) = Self::load().await {
            log::error!("[A2A] Failed to load agents: {}", e);
            exit(1)
        }
    }

    async fn load() -> anyhow::Result<()> {
        let agents = Self::fetch_agents().await?;
        log::info!("[A2A] Loaded {} agents", agents.len());
        AGENT_FACTORY.get_or_init(|| Self {
            agents: agents
                .into_iter()
                .map(|a| (a.name.clone(), a))
                .collect(),
        });
        Self::watch();
        Ok(())
    }

    async fn fetch_agents() -> anyhow::Result<Vec<Agent>> {
        AGENT_HTTP_CLIENT.fetch_agents().await
    }

    const INTERVAL: Duration = Duration::from_secs(5);

    fn watch() {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Self::INTERVAL);
            loop {
                interval.tick().await;
                let list = match Self::fetch_agents().await {
                    Ok(list) => list,
                    Err(e) => {
                        log::error!("[A2A] Fetch agents error: {}", e);
                        continue;
                    }
                };

                let old = AGENT_FACTORY.get().unwrap();
                let new_map: HashMap<String, Agent> = list
                    .into_iter()
                    .map(|a| (a.name.clone(), a))
                    .collect();

                // 移除已下线的 Agent
                old.agents.retain(|name, _| {
                    if !new_map.contains_key(name) {
                        log::info!("[A2A] Removed agent: {}", name);
                        return false;
                    }
                    true
                });

                // 处理新增和变更的 Agent
                for (name, new_agent) in new_map {
                    let need_update = match old.agents.get(&name) {
                        Some(existing) => *existing != new_agent,
                        None => true,
                    };
                    if need_update {
                        if old.agents.contains_key(&name) {
                            log::info!("[A2A] Updated agent: {}", name);
                        } else {
                            log::info!("[A2A] New agent registered: {}", name);
                        }
                        old.agents.insert(name, new_agent);
                    }
                }
            }
        });
    }

    /// 根据名称查找 Agent
    pub fn get_agent(name: &str) -> Option<Agent> {
        AGENT_FACTORY
            .get()
            .unwrap()
            .agents
            .get(name)
            .map(|r| r.clone())
    }
}
