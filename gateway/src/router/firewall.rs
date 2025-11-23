use crate::router::client::INNER_HTTP_CLIENT;
use anyhow::Context;
use protocol::gateway::{AllowDenyPolicy, Firewall};
use std::process::exit;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::RwLock;

pub struct Firewalld {
    pub config: Arc<RwLock<Firewall>>,
    hash: Arc<RwLock<String>>,
}

pub static FIREWALLD: OnceLock<Firewalld> = OnceLock::new();

impl Firewalld {
    pub async fn init() {
        if let Err(e) = Self::load().await {
            log::error!("{}", e);
            exit(1)
        }
    }

    async fn load() -> anyhow::Result<()> {
        let firewall = Self::fetch_firewall().await?;
        log::info!("loaded firewall config: {:?}", firewall);

        let hash = md5::compute(serde_json::to_string(&firewall)?);
        let hash = format!("{:x}", hash);

        FIREWALLD.get_or_init(|| Self {
            config: Arc::new(RwLock::new(firewall)),
            hash: Arc::new(RwLock::new(hash)),
        });

        Self::watch();

        Ok(())
    }

    async fn fetch_firewall() -> anyhow::Result<Firewall> {
        INNER_HTTP_CLIENT.fetch_firewall().await
    }

    const INTERVAL: Duration = Duration::from_secs(5);

    fn watch() {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Self::INTERVAL);
            loop {
                interval.tick().await;
                let config = match Self::fetch_firewall().await {
                    Ok(config) => config,
                    Err(e) => {
                        log::error!("{}", e);
                        continue;
                    }
                };

                let hash = md5::compute(
                    serde_json::to_string(&config)
                        .context("serialize firewall config")
                        .unwrap(),
                );
                let hash = format!("{:x}", hash);

                let old_config = FIREWALLD.get().unwrap();

                if *old_config.hash.read().await == hash {
                    log::debug!("gateway firewall not changed, wait next interval");
                    continue;
                }

                log::info!("loaded gateway firewall: {:?}", config);

                {
                    *old_config.config.write().await = config;
                    *old_config.hash.write().await = hash;
                }
            }
        });
    }
    pub async fn check(ip: &str, referer: &str) -> Result<(), String> {
        let firewall = FIREWALLD.get().unwrap().config.read().await;

        // 可信IP直接通过
        if firewall.trust_ips.contains(ip) {
            return Ok(());
        }

        // 检查IP策略
        match firewall.ip_policy_mode {
            AllowDenyPolicy::Allow => {
                if !firewall.ip_policy.contains(ip) {
                    return Err(format!("Your IP ({}) is not allowed", ip));
                }
            }
            AllowDenyPolicy::Deny => {
                if firewall.ip_policy.contains(ip) {
                    return Err(format!("Your IP ({}) is not allowed", ip));
                }
            }
            AllowDenyPolicy::Disable => {}
        }

        // 检查referer策略
        match firewall.referer_policy_mode {
            AllowDenyPolicy::Allow => {
                if referer.is_empty() && !firewall.allow_empty_referer {
                    return Err("Your referer is not allowed".to_string());
                }
                if !firewall.referer_policy.contains(referer) {
                    return Err("Your referer is not allowed".to_string());
                }
            }
            AllowDenyPolicy::Deny => {
                if referer.is_empty() && !firewall.allow_empty_referer {
                    return Err("Your referer is not allowed".to_string());
                }
                if firewall.referer_policy.contains(referer) {
                    return Err("Your referer is not allowed".to_string());
                }
            }
            AllowDenyPolicy::Disable => {}
        }

        // 检查最大连接数

        Ok(())
    }
}
