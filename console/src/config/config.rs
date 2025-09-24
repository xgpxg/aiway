use common::data_dir;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use logging::log;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 服务器配置
    #[serde(default = "ServerConfig::default")]
    server: ServerConfig,
    #[serde(default = "Mode::default")]
    mode: Mode,
    /// 数据库配置
    #[serde(default = "DatabaseConfig::default")]
    database: DatabaseConfig,
    /// 缓存配置
    #[serde(default = "RedisConfig::default")]
    redis: RedisConfig,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            mode: Mode::default(),
            database: DatabaseConfig::default(),
            redis: RedisConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "ServerConfig::default_address")]
    pub address: String,
    #[serde(default = "ServerConfig::default_port")]
    pub port: u16,
}

impl ServerConfig {
    fn default() -> Self {
        Self {
            address: Self::default_address(),
            port: Self::default_port(),
        }
    }
    fn default_address() -> String {
        "127.0.0.1".to_string()
    }
    fn default_port() -> u16 {
        6000
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialOrd, PartialEq)]
pub enum Mode {
    #[serde(rename = "standalone")]
    Standalone,
    #[serde(rename = "cluster")]
    Cluster,
}
impl Mode {
    fn default() -> Self {
        Mode::Standalone
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    #[serde(default = "DatabaseConfig::default_url")]
    pub url: String,
    #[serde(default = "DatabaseConfig::default_username")]
    pub username: String,
    #[serde(default = "DatabaseConfig::default_password")]
    pub password: String,
}
impl DatabaseConfig {
    pub fn default() -> Self {
        Self {
            url: Self::default_url(),
            username: Self::default_username(),
            password: Self::default_password(),
        }
    }

    pub fn default_url() -> String {
        let url = data_dir!("sqlite", "main.db");
        format!("sqlite://{}", url.display()).to_string()
    }

    pub fn default_username() -> String {
        "".to_string()
    }
    pub fn default_password() -> String {
        "".to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RedisConfig {
    #[serde(rename = "single")]
    Single {
        url: String,
        password: Option<String>,
    },
    #[serde(rename = "cluster")]
    Cluster {
        nodes: Vec<String>,
        password: Option<String>,
    },
}

impl RedisConfig {
    pub fn default() -> Self {
        Self::Single {
            url: "redis://127.0.0.1:6379".to_string(),
            password: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiConfig {
    pub token: String,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

pub struct AppConfig;
impl AppConfig {
    pub fn server() -> &'static ServerConfig {
        &CONFIG.get().expect("Config not initialized").server
    }
    pub fn mode() -> &'static Mode {
        &CONFIG.get().expect("Config not initialized").mode
    }
    pub fn database() -> &'static DatabaseConfig {
        &CONFIG.get().expect("Config not initialized").database
    }

    pub fn redis() -> &'static RedisConfig {
        &CONFIG.get().expect("Config not initialized").redis
    }

}

pub fn init(path: &str) -> anyhow::Result<()> {
    let path = std::path::Path::new(path);

    let config = if !path.exists() {
        log::info!("config file not set, use default config");
        Config::default()
    } else {
        serde_yaml::from_str::<Config>(&std::fs::read_to_string(path)?)?
    };

    let _ = CONFIG.set(config);
    Ok(())
}
