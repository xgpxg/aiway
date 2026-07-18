use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentEntryConfig {
    /// Agent 服务入口地址，如 "https://a2a.example.com" 或 "http://192.168.1.100:7001"
    pub entry_url: String,
}
