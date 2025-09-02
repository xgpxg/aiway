use crate::app::get_app;
use crate::config::server::ConfigEntry;
use crate::config::server::res::{PageRes, Res};
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

pub fn routes() -> Vec<rocket::Route> {
    routes![upsert, get, delete, recover, list, list_history]
}

#[derive(Debug, Serialize, Deserialize)]
struct UpsertConfigReq {
    namespace_id: String,
    id: String,
    content: String,
    description: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
struct DeleteConfigReq {
    namespace_id: String,
    id: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct RecoverConfigReq {
    id_: i64,
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

/// 删除配置
#[post("/delete", data = "<req>")]
async fn delete(req: Json<DeleteConfigReq>) -> Res<()> {
    match get_app()
        .config_app
        .manager
        .delete_config_and_sync(&req.namespace_id, &req.id)
        .await
    {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 恢复配置
#[post("/recover", data = "<req>")]
async fn recover(req: Json<RecoverConfigReq>) -> Res<()> {
    match get_app().config_app.manager.recovery(req.id_).await {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 获取配置列表
#[get("/list?<namespace_id>&<page_num>&<page_size>")]
async fn list(namespace_id: &str, page_num: i32, page_size: i32) -> Res<PageRes<ConfigEntry>> {
    match get_app()
        .config_app
        .manager
        .list_configs_with_page(namespace_id, page_num, page_size)
        .await
    {
        Ok(res) => Res::success(PageRes {
            page_num,
            page_size,
            total: res.0,
            list: res.1,
        }),
        Err(e) => Res::error(&e.to_string()),
    }
}

/// 获取配置历史列表
#[get("/histories?<namespace_id>&<id>&<page_num>&<page_size>")]
async fn list_history(
    namespace_id: &str,
    id: &str,
    page_num: i32,
    page_size: i32,
) -> Res<PageRes<ConfigEntry>> {
    match get_app()
        .config_app
        .manager
        .list_config_history_with_page(namespace_id, id, page_num, page_size)
        .await
    {
        Ok(res) => Res::success(PageRes {
            page_num,
            page_size,
            total: res.0,
            list: res.1,
        }),
        Err(e) => Res::error(&e.to_string()),
    }
}
