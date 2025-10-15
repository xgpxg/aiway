use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogListRes {
    /// 模糊搜索：日志内容
    pub filter_text: Option<String>,
}
