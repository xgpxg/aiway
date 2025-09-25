use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRes {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserCenterRes {
    pub id: i64,
    pub base_info: UserBaseInfo,
    pub other: OtherInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserBaseInfo {
    pub username: String,
    pub nickname: Option<String>,
    pub avatar: Option<String>,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OtherInfo {
    pub password_has_set: bool,
}
