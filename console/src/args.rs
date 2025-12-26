use clap::Parser;
use common::data_dir;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    // /// 配置文件，YAML格式
    // #[arg(short, long, default_value = "config.yaml")]
    // pub config: String,
    /// Listen address, like 127.0.0.1
    #[arg(short, long, default_value = "127.0.0.1")]
    pub address: String,

    /// Port
    #[arg(short, long, default_value_t = 7000)]
    pub port: u16,

    /// Database connection url
    #[arg(long, default_value_t = Args::default_db_url())]
    pub db_url: String,

    /// Database username
    #[arg(long, default_value = "")]
    pub db_username: String,

    /// Database password
    #[arg(long, default_value = "")]
    pub db_password: String,

    /// Log server address
    #[arg(long, default_value = "127.0.0.1:7280")]
    pub log_server: String,

    /// Cache connection url
    #[arg(long, default_value = "redis://127.0.0.1:6379")]
    pub cache_url: String,

    /// Cache username
    #[arg(long, default_value = "")]
    pub cache_username: String,

    /// Cache password
    #[arg(long, default_value = "")]
    pub cache_password: String,
}

impl Args {
    fn default_db_url() -> String {
        let url = data_dir!("sqlite", "main.db");
        format!("sqlite://{}", url.display()).to_string()
    }
}
