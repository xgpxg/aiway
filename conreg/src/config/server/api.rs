use crate::app::get_app;
use crate::config::server::ConfigEntry;
use crate::config::server::res::Res;
use rocket::State;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use logging::log;

pub fn routes() -> Vec<rocket::Route> {
    routes![upsert, get, read]
}

#[derive(Debug, Serialize, Deserialize)]
struct UpsertConfigReq {
    namespace_id: String,
    id: String,
    content: String,
    description: Option<String>,
}

/// 创建或更新配置
#[post("/upsert", data = "<req>")]
async fn upsert(req: Json<UpsertConfigReq>) -> Res<()> {
    match get_app()
        .config_app
        .manager
        .upsert_config_and_sync(
            &req.namespace_id,
            &req.id,
            &req.content,
            req.description.clone(),
        )
        .await
    {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 获取配置
///
/// 该接口仅用于后台获取配置，客户端获取配置请使用`read`接口
#[get("/get?<namespace_id>&<id>")]
async fn get(namespace_id: &str, id: &str) -> Res<Option<ConfigEntry>> {
    match get_app()
        .config_app
        .manager
        .get_config(namespace_id, id)
        .await
    {
        Ok(entry) => Res::success(entry),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 读取配置内容
#[get("/read?<namespace_id>&<id>")]
async fn read(namespace_id: &str, id: &str) -> Option<String> {
    match get_app()
        .config_app
        .manager
        .get_config(namespace_id, id)
        .await
    {
        Ok(entry) => match entry {
            Some(entry) => Some(entry.content),
            None => None,
        },
        Err(e) => {
            log::error!("{}", e);
            None
        }
    }
}
