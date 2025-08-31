use crate::config::RaftApp;
use crate::config::raft::declare_types::ClientWriteResponse;
use crate::config::raft::{App, RaftRequest, TypeConfig};
use openraft::error::CheckIsLeaderError;
use openraft::error::Infallible;
use openraft::error::decompose::DecomposeResult;
use rocket::http::Status;
use rocket::response::Responder;
use rocket::serde::json::Json;
use rocket::{State, post};
use logging::log;

/**
 * Application API
 *
 * This is where you place your application, you can use the example below to create your
 * API. The current implementation:
 *
 *  - `POST - /write` saves a value in a key and sync the nodes.
 *  - `POST - /read` attempt to find a value from a given key.
 */
#[post("/write", data = "<req>")]
pub async fn write(
    app: &State<RaftApp>,
    req: Json<RaftRequest>,
) -> Result<Json<ClientWriteResponse>, Status> {
    match app.raft.client_write(req.0).await {
        Ok(response) => Ok(Json(response)),
        Err(err) => {
            log::error!("Error writing: {}", err);
            Err(Status::InternalServerError)
        }
    }
}

#[get("/read?<key>")]
pub async fn read(app: &State<App>, key: &str) -> Json<String> {
    let state_machine = &app.state_machine;
    match state_machine.read().await.data.get(key).cloned() {
        Some(value) => Json(value),
        None => Json("".to_string()),
    }
}
