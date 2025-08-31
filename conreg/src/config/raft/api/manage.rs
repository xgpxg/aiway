use crate::config::RaftApp;
use crate::config::raft::api::{ForwardRequest, forward_request_to_leader};
use crate::config::raft::declare_types::{Node, RaftMetrics};
use crate::config::raft::{NodeId, TypeConfig};
use logging::log;
use openraft::error::{ClientWriteError, RaftError};
use openraft::raft::ClientWriteResponse;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{State, get, post};
use std::collections::BTreeMap;
use std::collections::BTreeSet;

/// 初始化集群
///
/// 当请求中没有传集群信息时，默认初始化当前接节点为单实例集群，
/// 后续可通过`add_learner`添加
///
/// 示例：`curl -X POST http://127.0.0.1:8000/init -d []`
#[post("/init", data = "<req>")]
pub async fn init(
    app: &State<RaftApp>,
    req: Json<Vec<(NodeId, String)>>,
) -> Result<Json<()>, Status> {
    let mut nodes = BTreeMap::new();
    if req.0.is_empty() {
        nodes.insert(
            app.id,
            Node {
                addr: app.addr.clone(),
            },
        );
    } else {
        for (id, addr) in req.0.into_iter() {
            nodes.insert(id, Node { addr });
        }
    };
    match app.raft.initialize(nodes).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            log::error!("{}", e);
            Err(Status::InternalServerError)
        }
    }
}

/// 添加一个Learner节点
///
/// Learner节点接收主节点的日志，但不参与投票。
/// 也就是说，Learner能够响应读请求，但是无法将自己转为Leader节点，
/// 要转为Follower节点，需要调用`change-membership`来改变集群成员配置。
///
/// 示例：`curl -X POST http://localhost:8000/add-learner -d '[2,"127.0.0.1:8001"]'`
#[post("/add-learner", data = "<req>")]
pub async fn add_learner(
    app: &State<RaftApp>,
    req: Json<(NodeId, String)>,
) -> Result<Json<ClientWriteResponse<TypeConfig>>, Status> {
    let (node_id, api_addr) = req.0;
    let node = Node { addr: api_addr };
    match app.raft.add_learner(node_id, node, true).await {
        Ok(response) => Ok(Json(response)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// 添加或删除集群节点
///
/// 示例：`curl -X POST http://localhost:8000/change-membership -d '[1,2,3]'`
#[post("/change-membership", data = "<req>")]
pub async fn change_membership(
    app: &State<RaftApp>,
    req: Json<BTreeSet<NodeId>>,
) -> Result<Json<ClientWriteResponse<TypeConfig>>, Status> {
    match app.raft.change_membership(req.0.clone(), false).await {
        Ok(res) => Ok(Json(res)),
        Err(e) => {
            match e {
                RaftError::APIError(err) => match err {
                    ClientWriteError::ForwardToLeader(fl) => match fl.leader_node {
                        Some(node) => {
                            log::debug!(
                                "forward to leader {}, leader address: {}",
                                fl.leader_id.unwrap(),
                                node.addr
                            );
                            return forward_request_to_leader(
                                &node.addr,
                                ForwardRequest::MembershipRequest(req.into_inner()),
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

/// 获取集群信息
///
/// 示例：`curl -X GET http://localhost:8000/metrics`
#[get("/metrics")]
pub async fn metrics(app: &State<RaftApp>) -> Result<Json<RaftMetrics>, Status> {
    let metrics = app.raft.metrics().borrow().clone();
    Ok(Json(metrics))
}
