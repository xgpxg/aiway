use protocol::logg::{LogEntry, LogSearchReq, LogSearchRes};
use rocket::data::{ByteUnit, FromData, Outcome};
use rocket::serde::json::Json;
use rocket::{Data, Request, State, async_trait, post, routes};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tantivy::collector::TopDocs;
use tantivy::directory::MmapDirectory;
use tantivy::query::QueryParser;
use tantivy::schema::document::CompactDocValue;
use tantivy::schema::{
    DateOptions, DateTimePrecision, Field, IndexRecordOption, STORED, Schema, TEXT,
    TextFieldIndexing, TextOptions, Value,
};
use tantivy::tokenizer::{LowerCaser, TextAnalyzer};
use tantivy::{
    DateTime, Document, Index, IndexReader, IndexWriter, Order, ReloadPolicy, TantivyDocument,
    TantivyError,
};

struct Fields {
    time: Field,
    service: Field,
    level: Field,
    message: Field,
}

impl Fields {
    fn from_schema(schema: &Schema) -> Self {
        Fields {
            time: schema.get_field("time").unwrap(),
            service: schema.get_field("service").unwrap(),
            level: schema.get_field("level").unwrap(),
            message: schema.get_field("message").unwrap(),
        }
    }
}

pub(crate) struct Logg {
    index: Index,
    fields: Fields,
    index_writer: Arc<Mutex<IndexWriter>>,
    reader: IndexReader,
}

impl Logg {
    const MEMORY_BUDGET_IN_BYTES: usize = 32 * 1024 * 1024;
    const FLUSH_INTERVAL: Duration = Duration::from_secs(1);
    const TIME_FORMAT: &'static str = "%Y-%m-%d %H:%M:%S%.3f";
    const TIME_OFFSET: i64 = 8 * 3600;
    pub(crate) fn new(dir: &str) -> Result<Self, TantivyError> {
        let index = Self::open_or_create_index(dir)?;
        // 添加jie_ba分词器
        Self::register_tokenizer(&index);
        let schema = index.schema();
        let index_writer = index.writer(Self::MEMORY_BUDGET_IN_BYTES)?;
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?;

        Ok(Self {
            index,
            fields: Fields::from_schema(&schema),
            index_writer: Arc::new(Mutex::new(index_writer)),
            reader,
        })
    }
    fn open_or_create_index(dir: &str) -> Result<Index, TantivyError> {
        let mut sb = Schema::builder();
        sb.add_date_field(
            "time",
            DateOptions::default()
                .set_fast()
                .set_precision(DateTimePrecision::Microseconds)
                | STORED,
        );
        sb.add_text_field("service", TEXT | STORED);
        sb.add_text_field("level", TEXT | STORED);
        sb.add_text_field(
            "message",
            TextOptions::default()
                .set_indexing_options(
                    TextFieldIndexing::default()
                        .set_tokenizer("jie_ba")
                        .set_index_option(IndexRecordOption::WithFreqsAndPositions),
                )
                .set_stored()
                | STORED,
        );

        let schema = sb.build();

        if !Path::new(dir).exists() {
            fs::create_dir_all(dir)?;
        }
        Index::open_or_create(MmapDirectory::open(dir)?, schema)
    }

    fn register_tokenizer(index: &Index) {
        let tokenizer = tantivy_jieba::JiebaTokenizer {};
        let analyzer = TextAnalyzer::builder(tokenizer)
            //.filter(RemoveLongFilter::limit(40))
            .filter(LowerCaser)
            .build();
        index.tokenizers().register("jie_ba", analyzer);
    }

    // 时间字符串转tantivy支持的时间类型
    fn string_to_datetime(date_str: &str) -> Result<DateTime, Box<dyn std::error::Error>> {
        use chrono::{NaiveDateTime, TimeZone, Utc};

        let naive_dt = NaiveDateTime::parse_from_str(date_str, Self::TIME_FORMAT)?;

        let utc_dt = Utc.from_utc_datetime(&naive_dt);

        let millis = utc_dt.timestamp_millis();

        Ok(DateTime::from_timestamp_millis(millis))
    }

    pub fn add(&self, entries: Vec<LogEntry>) {
        let mut index_writer = self.index_writer.lock().unwrap();
        entries.into_iter().for_each(|entry| {
            let mut doc = TantivyDocument::default();
            doc.add_date(
                self.fields.time,
                Self::string_to_datetime(&entry.time).unwrap(),
            );
            doc.add_text(self.fields.service, entry.service);
            doc.add_text(self.fields.level, entry.level);
            doc.add_text(self.fields.message, entry.message);

            let _ = index_writer.add_document(doc);
        });
        index_writer.commit().unwrap();
    }

    pub fn search(&self, req: LogSearchReq) -> anyhow::Result<LogSearchRes<LogEntry>> {
        let schema = self.index.schema();
        let query_parser =
            QueryParser::for_index(&self.index, schema.fields().map(|(f, _)| f).collect());

        let mut query = vec![];

        if let Some(q) = req.query {
            if !q.is_empty() {
                query.push(q);
            }
        }

        if let Some(start_timestamp) = req.start_timestamp {
            query.push(format!(
                "time:>={:?}",
                DateTime::from_timestamp_secs(start_timestamp /* + Self::TIME_OFFSET*/)
            ));
        }

        if let Some(end_timestamp) = req.end_timestamp {
            query.push(format!(
                "time:<{:?}",
                DateTime::from_timestamp_secs(end_timestamp /* + Self::TIME_OFFSET*/)
            ));
        }

        if query.len() == 0 {
            query.push("*".to_string());
        }

        //println!("query: {}", query.join(" AND "));

        let query = query_parser.parse_query(&query.join(" AND "))?;

        let searcher = self.reader.searcher();
        let num_hits = query.count(&searcher)?;
        if num_hits == 0 {
            return Ok(LogSearchRes::default());
        }

        let top_docs: Vec<(DateTime, _)> = searcher.search(
            &query,
            &TopDocs::with_limit(req.max_hits)
                .and_offset(req.start_offset)
                .order_by_fast_field("time", Order::Desc),
        )?;

        let mut list = Vec::new();

        let get_first_value =
            |value: CompactDocValue| -> String { value.as_str().unwrap_or_default().to_string() };
        let get_first_datetime_value = |value: CompactDocValue| -> String {
            value
                .as_datetime()
                .map(|dt| {
                    use chrono::{TimeZone, Utc};
                    let dt = Utc
                        .timestamp_millis_opt(dt.into_timestamp_millis())
                        .unwrap();
                    dt.format(Self::TIME_FORMAT).to_string()
                })
                .unwrap_or_default()
        };
        for (_score, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;
            let mut log_entry = LogEntry::default();
            for (field, value) in retrieved_doc.iter_fields_and_values() {
                match field.field_id() {
                    fid if fid == self.fields.time.field_id() => {
                        log_entry.time = get_first_datetime_value(value);
                    }
                    fid if fid == self.fields.service.field_id() => {
                        log_entry.service = get_first_value(value);
                    }
                    fid if fid == self.fields.level.field_id() => {
                        log_entry.level = get_first_value(value);
                    }
                    fid if fid == self.fields.message.field_id() => {
                        log_entry.message = get_first_value(value);
                    }
                    _ => {}
                }
            }
            list.push(log_entry);
        }

        Ok(LogSearchRes {
            num_hits,
            hits: list,
            aggregations: None,
        })
    }
}

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
#[post("/ingest", data = "<req>")]
fn ingest(req: LogEntries, logg: &State<Logg>) {
    logg.add(req.0);
}

/// 查询日志
#[post("/search", data = "<req>")]
fn search(req: Json<LogSearchReq>, logg: &State<Logg>) -> Json<LogSearchRes<LogEntry>> {
    match logg.search(req.0) {
        Ok(res) => Json(res),
        Err(e) => {
            println!("Error: {}", e);
            Json(LogSearchRes::default())
        }
    }
}
