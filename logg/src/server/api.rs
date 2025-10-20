use crate::server::{LogEntry, LogSearchRes, Logg};
use protocol::logg::LogSearchReq;
use rocket::data::{ByteUnit, FromData, Outcome};
use rocket::serde::Serialize;
use rocket::serde::json::Json;
use rocket::{Data, Request, State, async_trait, get, post, routes};

#[derive(Debug)]
struct LogEntries(Vec<LogEntry>);

#[async_trait]
impl<'r> FromData<'r> for LogEntries {
    type Error = ();

    async fn from_data(_req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
        let bytes = data.open(ByteUnit::Mebibyte(8)).into_bytes().await.unwrap();
        let lines = String::from_utf8(bytes.value).unwrap();
        let lines = lines.lines();

        let entries = lines
            .map(|line| serde_json::from_str(&line).unwrap())
            .collect::<Vec<_>>();

        Outcome::Success(LogEntries(entries))
    }
}

pub fn routes() -> Vec<rocket::Route> {
    routes![ingest, search]
}

/// 添加日志
#[post("/<index>/ingest", data = "<req>")]
fn ingest(index: &str, req: LogEntries, logg: &State<Logg>) {
    logg.add(req.0);
}

/// 查询日志
#[post("/<index>/search", data = "<req>")]
fn search(index: &str, req: Json<LogSearchReq>, logg: &State<Logg>) -> Json<LogSearchRes> {
    match logg.search(req.0) {
        Ok(res) => Json(res),
        Err(e) => {
            println!("Error: {}", e);
            Json(LogSearchRes::default())
        }
    }
}
