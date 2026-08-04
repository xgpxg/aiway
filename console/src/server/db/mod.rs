use crate::args::Args;
use anyhow::bail;
use logging::log;
use rbatis::RBatis;
use std::sync::OnceLock;

mod migrations;
pub mod models;
mod mysql;
mod sqlite;
pub mod tools;

static RB: OnceLock<RBatis> = OnceLock::new();

pub struct Pool;
impl Pool {
    pub fn get<'a>() -> anyhow::Result<&'a RBatis> {
        match RB.get() {
            None => {
                log::error!("rbatis not init");
                bail!("rbatis not init".to_string());
            }
            Some(rb) => Ok(rb),
        }
    }
}

pub async fn init(args: &Args) -> anyhow::Result<()> {
    let url = args.db_url.as_str();
    match url {
        url if url.starts_with("sqlite") => sqlite::init(url).await,
        url if url.starts_with("mysql") => {
            mysql::init(url, &args.db_username, &args.db_password).await
        }
        _ => bail!("database not support"),
    };

    let rb = Pool::get().expect("database not initialized");
    if let Err(e) = migrations::run_all(rb).await {
        log::error!("migration error: {}", e);
        bail!("migration error: {}", e);
    }

    Ok(())
}

#[macro_export]
macro_rules! update_nullable_fields {
    ($tx:expr, $table_name:expr, $id:expr, $($field:ident = $value:expr),* ) => {
        $(
            if $value.is_none() {
                $tx.exec(
                    &format!("update {} set {} = null where id = ?", $table_name, stringify!($field)),
                    vec![value!($id)],
                )
                .await?;
            }
        )*
    };
}
