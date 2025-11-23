use crate::server::db::{Pool, RB};
use anyhow::bail;
use logging::log;
use rbatis::RBatis;
use rbatis::executor::RBatisTxExecutor;
use rbdc_pool_fast::FastPool;
use rbdc_sqlite::{SqliteConnectOptions, SqliteDriver};
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::process::exit;
use std::str::FromStr;

pub(crate) async fn init(url: &str) {

    let rb = RBatis::new();
    let opts = SqliteConnectOptions::from_str(url).unwrap();
    if let Err(e) =
        rb.init_option::<SqliteDriver, SqliteConnectOptions, FastPool>(SqliteDriver {}, opts)
    {
        log::error!("db init error: {}", e);
        exit(1);
    }

    rb.exec(include_str!("sql/init.sql"), vec![])
        .await
        .map_err(|e| {
            log::error!("db init error: {}", e);
            exit(1);
        })
        .unwrap();

    log::info!("sqlite init success");
    RB.get_or_init(|| rb);
}

/// 执行事务闭包
/// - exec：闭包，返回Ok则提交事务，否则回滚
#[allow(unused)]
pub async fn tx<F, R, RV>(exec: F) -> anyhow::Result<RV>
where
    F: Fn(RBatisTxExecutor) -> R,
    R: Future<Output = anyhow::Result<RV>>,
{
    let tx = match Pool::get()?.acquire_begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("事务异常: {}", e);
            bail!(e);
        }
    };

    let result = exec(tx.clone()).await;

    match result {
        Ok(result) => {
            match tx.commit().await {
                Ok(_) => log::debug!("事务提交成功，事务ID：{}", tx.tx_id),
                Err(e) => {
                    log::error!("事务提交失败，事务ID：{}， 原因： {}", tx.tx_id, e);
                    bail!(e);
                }
            };
            Ok(result)
        }
        Err(e) => {
            log::debug!("事务闭包执行失败，即将回滚，错误原因: {}", e);
            match tx.rollback().await {
                Ok(_) => {
                    log::debug!("事务回滚成功，事务ID：{}", tx.tx_id);
                    Err(e)
                }
                Err(e) => {
                    log::error!("事务回滚失败，事务ID：{}， 原因： {}", tx.tx_id, e);
                    bail!(e);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(unused)]
pub struct Count {
    pub count: usize,
}

impl Deref for Count {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.count
    }
}
