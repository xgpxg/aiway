use crate::config::config::DatabaseConfig;
use crate::server::db::RB;
use rbatis::RBatis;
use rbdc_mysql::MysqlDriver;
use rbdc_mysql::options::MySqlConnectOptions;
use rbdc_pool_fast::FastPool;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::process::exit;
use std::str::FromStr;
use logging::log;

pub(crate) async fn init(config: &DatabaseConfig) {
    let db_url = config.url.as_str();

    let rb = RBatis::new();
    let opts = MySqlConnectOptions::from_str(db_url)
        .unwrap()
        .username(config.username.as_str())
        .password(config.password.as_str());
    if let Err(e) =
        rb.init_option::<MysqlDriver, MySqlConnectOptions, FastPool>(MysqlDriver {}, opts)
    {
        log::error!("db init error: {}", e);
        exit(1);
    }

    rb.exec(include_str!("sql/init.mysql.sql"), vec![])
        .await
        .map_err(|e| {
            log::error!("db init error: {}", e);
            exit(1);
        })
        .unwrap();

    log::info!("mysql init success");
    RB.get_or_init(|| rb);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Count {
    pub count: usize,
}

impl Deref for Count {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.count
    }
}
