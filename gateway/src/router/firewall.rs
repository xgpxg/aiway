use crate::router::GATEWAY_CONFIG;
use protocol::gateway::AllowDenyPolicy;

pub struct Firewalld {}
impl Firewalld {
    pub async fn check(ip: &str, referer: &str) -> Result<(), String> {
        let config = GATEWAY_CONFIG.get().unwrap().config.read().await;
        let firewall = &config.firewall;

        // 检查IP策略
        match firewall.ip_policy_mode {
            AllowDenyPolicy::Allow => {
                if !firewall.ip_policy.contains(ip) {
                    return Err("Your IP is not allowed".to_string());
                }
            }
            AllowDenyPolicy::Deny => {
                if firewall.ip_policy.contains(ip) {
                    return Err("Your IP is not allowed".to_string());
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
