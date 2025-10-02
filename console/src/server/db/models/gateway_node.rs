use derive_builder::Builder;
use rbatis::crud;
use rbatis::rbdc::DateTime;
use rocket::serde::{Deserialize, Serialize};

/// 网关节点信息
#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
#[builder(default)]
pub struct GatewayNode {
    pub id: Option<i64>,
    /// 节点ID
    pub node_id: Option<String>,
    /// 节点名称
    pub node_name: Option<String>,
    ///  IP
    pub ip: Option<String>,
    /// 端口
    pub port: Option<u16>,
    /// 节点状态
    pub status: Option<GatewayNodeStatus>,
    /// 节点状态信息
    pub status_msg: Option<String>,
    /// 最后一次心跳时间
    pub last_heartbeat_time: Option<DateTime>,
    /// 创建人ID
    pub create_user_id: Option<i64>,
    /// 修改人ID
    pub update_user_id: Option<i64>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
    /// 备注
    pub remark: Option<String>,
    /// 是否删除
    pub is_delete: Option<i8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GatewayNodeStatus {
    Online,
    Offline,
    Unknown,
}

crud!(GatewayNode {});
