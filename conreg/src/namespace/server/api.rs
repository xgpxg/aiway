use crate::app::get_app;
use crate::protocol::res::Res;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

pub fn routes() -> Vec<rocket::Route> {
    routes![upsert, delete]
}

#[derive(Debug, Serialize, Deserialize)]
struct UpsertConfigReq {
    id: String,
    name: String,
    description: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
struct DeleteConfigReq {
    id: String,
}

#[post("/upsert", data = "<req>")]
pub async fn upsert(req: Json<UpsertConfigReq>) -> Res<()> {
    match get_app()
        .namespace_app
        .manager
        .upsert_namespace_and_sync(&req.id, &req.name, req.description.clone())
        .await
    {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}

#[post("/delete", data = "<req>")]
pub async fn delete(req: Json<DeleteConfigReq>) -> Res<()> {
    match get_app()
        .namespace_app
        .manager
        .delete_namespace_and_sync(&req.id)
        .await
    {
        Ok(_) => Res::success(()),
        Err(e) => Res::error(&e.to_string()),
    }
}
