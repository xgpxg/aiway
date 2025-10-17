use crate::server::{LogEntry, Logg};
use rocket::data::{ByteUnit, FromData, Outcome};
use rocket::serde::json::Json;
use rocket::{Data, Request, State, async_trait, post, routes};

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
    routes![ingest]
}

#[post("/<index>/ingest", data = "<req>")]
fn ingest(index: &str, req: LogEntries, logg: &State<Logg>) {
    logg.add(req.0);
}
