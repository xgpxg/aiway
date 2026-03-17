use crate::server::db::Pool;
use chrono::Utc;
use logging::log;

/// 间隔180天
pub const API_KEY_CLEAN_INTERVAL: i64 = 180 * 24 * 60 * 60 * 1000;
pub(crate) async fn clean_api_key() {
    if let Err(e) = clean_api_key_().await {
        log::error!("clean api key error: {}", e);
    }
}

/// APIKey定时清理触发条件（同时满足）：
/// - `is_delete = 1`
/// - `now - ts > 定时清理时间间隔 `
async fn clean_api_key_() -> anyhow::Result<()> {
    let now = Utc::now().timestamp_millis();
    let sql = format!(
        "delete from api_key where is_delete = 1 and ts < {}",
        now - API_KEY_CLEAN_INTERVAL
    );

    let tx = Pool::get()?;
    let result = tx.exec(&sql, vec![]).await?;
    log::info!("clean {} deleted api keys", result.rows_affected);
    Ok(())
}
