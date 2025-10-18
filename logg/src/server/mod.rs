mod api;

use crate::Args;
use rocket::Config;
use rocket::data::{ByteUnit, FromData, Limits};
use rocket::serde::Serialize;
use serde::Deserialize;
use std::fmt::Debug;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tantivy::collector::TopDocs;
use tantivy::directory::MmapDirectory;
use tantivy::query::{Query, QueryParser};
use tantivy::schema::document::CompactDocValue;
use tantivy::schema::{
    DateOptions, DateTimePrecision, Field, IndexRecordOption, STORED, Schema, TEXT,
    TextFieldIndexing, TextOptions, Value,
};
use tantivy::tokenizer::{LowerCaser, TextAnalyzer};
use tantivy::{
    DateTime, Document, Index, IndexWriter, Order, ReloadPolicy, Searcher, TantivyDocument,
    TantivyError,
};

pub async fn start_http_server(args: &Args) -> anyhow::Result<()> {
    let mut builder = rocket::build().configure(Config {
        port: args.port,
        limits: Limits::default()
            .limit("json", ByteUnit::Mebibyte(3))
            .limit("data-form", ByteUnit::Mebibyte(100))
            .limit("file", ByteUnit::Mebibyte(100)),
        log_level: rocket::config::LogLevel::Critical,
        cli_colors: false,
        ..Config::debug_default()
    });

    builder = builder.mount("/api/v1", api::routes());

    builder = builder.manage(Logg::new("logs/logs")?);

    builder.launch().await?;

    Ok(())
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LogEntry {
    time: String,
    service: String,
    level: String,
    message: String,
}

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

#[derive(Debug, Serialize, Default)]
pub struct LogListRes {
    num_hits: usize,
    hits: Vec<LogEntry>,
}

struct Logg {
    index: Index,
    fields: Fields,
    index_writer: Arc<Mutex<IndexWriter>>,
    searcher: Searcher,
}

impl Logg {
    const MEMORY_BUDGET_IN_BYTES: usize = 32 * 1024 * 1024;
    const FLUSH_INTERVAL: Duration = Duration::from_secs(1);
    const TIME_FORMAT: &'static str = "%Y-%m-%d %H:%M:%S%.3f";
    fn new(dir: &str) -> Result<Self, TantivyError> {
        let index = Self::open_or_create_index(dir)?;
        // 添加jie_ba分词器
        Self::register_tokenizer(&index);
        let schema = index.schema();
        let index_writer = index.writer(Self::MEMORY_BUDGET_IN_BYTES)?;
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?;
        let searcher = reader.searcher();

        Ok(Self {
            index,
            fields: Fields::from_schema(&schema),
            index_writer: Arc::new(Mutex::new(index_writer)),
            searcher,
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
        entries.into_iter().for_each(|entry| {
            let mut doc = TantivyDocument::default();
            doc.add_date(
                self.fields.time,
                Self::string_to_datetime(&entry.time).unwrap(),
            );
            doc.add_text(self.fields.service, entry.service);
            doc.add_text(self.fields.level, entry.level);
            doc.add_text(self.fields.message, entry.message);

            let _ = self.index_writer.lock().unwrap().add_document(doc);
        });
        self.index_writer.lock().unwrap().commit().unwrap();
    }

    pub fn search(&self, query: &str) -> anyhow::Result<LogListRes> {
        let schema = self.index.schema();
        let query_parser =
            QueryParser::for_index(&self.index, schema.fields().map(|(f, _)| f).collect());
        let query = query_parser.parse_query(query)?;

        let num_hits = query.count(&self.searcher)?;
        if num_hits == 0 {
            return Ok(LogListRes::default());
        }

        let top_docs: Vec<(DateTime, _)> = self.searcher.search(
            &query,
            &TopDocs::with_limit(10).order_by_fast_field("time", Order::Desc),
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
            let retrieved_doc: TantivyDocument = self.searcher.doc(doc_address)?;
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

        Ok(LogListRes {
            num_hits,
            hits: list,
        })
    }
}
