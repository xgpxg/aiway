use crate::server::db::Pool;
use crate::server::db::models::api_key;
use crate::server::db::models::api_key::ApiKey;
use crate::server::task::API_KEY_CLEAN_INTERVAL;
use aiway_protocol::gateway::{Action, ApiKeySync};
use logging::log;

/// 拉取APIKey，含逻辑删除的。
///
/// - 当 `last_pull_time = Some(t)` 时，拉取其之后的；
/// - 当 `last_pull_time = None` 时，全量拉取
///
/// 需要全量拉取的场景：
/// - 网关节点第一次启动，尚未拉取过
/// - 当 `now - last_pull_time >= 定时任务清理时间间隔`时
///
/// APIKey定时清理触发条件（同时满足）：
/// - `is_delete = 1`
/// - `now - ts > 定时清理时间间隔 `
pub(crate) async fn pull_api_key(last_pull_time: Option<i64>) -> anyhow::Result<Vec<ApiKeySync>> {
    let list = if let Some(last_pull_time) = last_pull_time
        && chrono::Utc::now().timestamp_millis() - last_pull_time < API_KEY_CLEAN_INTERVAL
    {
        api_key::list_by_update_time(Pool::get()?, last_pull_time).await?
    } else {
        log::info!("no last_pull_time from gateway, performing full APIKey fetch");
        ApiKey::select_all(Pool::get()?).await?
    };

    let list = list
        .into_iter()
        .map(|item| {
            let action = if item.is_delete == Some(1) {
                Action::Delete
            } else {
                if item.update_time.is_none() {
                    Action::Create
                } else {
                    Action::Update
                }
            };
            ApiKeySync {
                secret: item.secret.unwrap(),
                action,
                exp_time: item.exp_time.map(|item| item.unix_timestamp()),
            }
        })
        .collect::<Vec<_>>();

    if !list.is_empty() {
        log::info!(
            "pulled {} APIKeys since {}",
            list.len(),
            last_pull_time.unwrap_or(0)
        );
    }

    Ok(list)
}
