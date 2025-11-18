use crate::server::message::{MessageCountRes, MessageListReq};
use derive_builder::Builder;
use protocol::gateway::alert::AlertLevel;
use rbatis::executor::Executor;
use rbatis::rbdc::DateTime;
use rbatis::{crud, htmlsql, htmlsql_select_page};
use rocket::serde::{Deserialize, Serialize};

/// 消息通知
#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
#[builder(default)]
pub struct Message {
    pub id: Option<i64>,
    /// 级别：info | warn | error
    pub level: Option<AlertLevel>,
    /// 标题
    pub title: Option<String>,
    /// 内容
    pub content: Option<String>,
    /// 已读状态
    pub read_status: Option<MessageReadStatus>,
    #[serde(serialize_with = "crate::server::common::serialize_datetime")]
    pub create_time: Option<DateTime>,
    /// 是否删除
    pub is_delete: Option<i8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum MessageReadStatus {
    /// 未读
    #[default]
    Unread,
    /// 已读
    Read,
}

crud!(Message {});
htmlsql_select_page!(list_page(param: &MessageListReq) -> Message => "src/server/db/mapper/message.html");
htmlsql!(count_unread(rb: &dyn Executor) -> MessageCountRes => "src/server/db/mapper/message.html");
