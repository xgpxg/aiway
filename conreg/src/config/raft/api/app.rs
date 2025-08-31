use crate::config::RaftApp;
use crate::config::raft::api::{ForwardRequest, forward_request_to_leader};
use crate::config::raft::declare_types::ClientWriteResponse;
use crate::config::raft::{App, RaftRequest};
use logging::log;
use openraft::error::{ClientWriteError, RaftError};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{State, post};

/// 写入数据
#[post("/write", data = "<req>")]
pub async fn write(
    app: &State<RaftApp>,
    req: Json<RaftRequest>,
) -> Result<Json<ClientWriteResponse>, Status> {
    match app.raft.client_write(req.0.clone()).await {
        Ok(response) => Ok(Json(response)),
        Err(err) => {
            match err {
                RaftError::APIError(err) => match err {
                    // 节点不是leader，Raft会返回一个需要转发到Leader的错误，需要手动处理下
                    ClientWriteError::ForwardToLeader(fl) => match fl.leader_node {
                        Some(node) => {
                            log::debug!(
                                "forward to leader {}, leader address: {}",
                                fl.leader_id.unwrap(),
                                node.addr
                            );
                            return forward_request_to_leader(
                                &node.addr,
                                ForwardRequest::RaftRequest(req.into_inner()),
                            )
                            .await;
                        }
                        None => {
                            log::debug!("forward to leader error: no leader");
                            return Err(Status::InternalServerError);
                        }
                    },
                    ClientWriteError::ChangeMembershipError(e) => {
                        log::error!("error when change membership: {:?}", e);
                    }
                },
                RaftError::Fatal(e) => {
                    log::error!("error when write: {:?}", e);
                }
            }
            Err(Status::InternalServerError)
        }
    }
}

/// 读取数据
#[get("/read?<key>")]
pub async fn read(app: &State<App>, key: &str) -> Json<Option<String>> {
    let state_machine = &app.state_machine;
    match state_machine.read().await.data.get(key).cloned() {
        Some(value) => Json(Some(value)),
        None => Json(None),
    }
}
