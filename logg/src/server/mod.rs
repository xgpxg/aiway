mod api;

use crate::Args;
use rocket::Config;
use rocket::data::{ByteUnit, FromData, Limits};
use rocket::serde::Serialize;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tantivy::directory::MmapDirectory;
use tantivy::schema::{
    DateOptions, Field, IndexRecordOption, STORED, Schema, TEXT, TextFieldIndexing, TextOptions,
};
use tantivy::tokenizer::{LowerCaser, TextAnalyzer};
use tantivy::{DateTime, Index, IndexWriter, TantivyDocument, TantivyError};

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

#[derive(Debug, Serialize, Deserialize)]
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

struct Logg {
    index: Index,
    fields: Fields,
    index_writer: Arc<Mutex<IndexWriter>>,
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
        Ok(Self {
            index,
            fields: Fields::from_schema(&schema),
            index_writer: Arc::new(Mutex::new(index_writer)),
        })
    }
    fn open_or_create_index(dir: &str) -> Result<Index, TantivyError> {
        let mut sb = Schema::builder();
        sb.add_date_field("time", DateOptions::default() | STORED);
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
}
