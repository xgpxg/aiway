use rocket::Request;
use rocket::response::Responder;
use serde::{Deserialize, Serialize};
use serde_json::json;

///通用Json响应返回
#[derive(Debug, Serialize, Deserialize)]
pub struct Res<T> {
    pub code: i32,
    pub msg: String,
    pub data: Option<T>,
}

/// 响应成功
const SUCCESS_CODE: i32 = 0;
/// 系统错误
const ERROR_CODE: i32 = 1;

impl<T> Res<T>
where
    T: Serialize,
{
    pub fn success(data: T) -> Self {
        Res {
            code: SUCCESS_CODE,
            msg: "".to_string(),
            data: Some(data),
        }
    }

    pub fn error(msg: &str) -> Self {
        Res {
            code: ERROR_CODE,
            msg: msg.to_string(),
            data: None,
        }
    }

    #[allow(unused)]
    pub fn is_success(&self) -> bool {
        self.code == 0
    }

    #[allow(unused)]
    pub fn to_json_string(&self) -> String {
        json!(&self).to_string()
    }
}

#[allow(unused)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageRes<T> {
    pub page_num: i32,
    pub page_size: i32,
    pub total: u64,
    pub list: Vec<T>,
}

#[allow(unused)]
pub trait IntoPageRes<I, T>
where
    I: Send + Sync,
    T: Send + Sync,
{
    fn convert_to_page_res<F>(self, f: F) -> PageRes<T>
    where
        F: Fn(Vec<I>) -> Vec<T>;
}

impl<'r, 'o: 'r, T: Serialize> Responder<'r, 'o> for Res<T> {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'o> {
        json!(&self).respond_to(request)
    }
}
