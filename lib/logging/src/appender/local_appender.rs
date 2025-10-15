use crate::appender::LogEntry;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tantivy::directory::MmapDirectory;
use tantivy::schema::{
    DateOptions, Field, IndexRecordOption, STORED, Schema, TEXT, TextFieldIndexing, TextOptions,
};
use tantivy::tokenizer::{LowerCaser, TextAnalyzer};
use tantivy::{DateTime, Document, Index, IndexWriter, TantivyDocument};
use tokio::sync::Notify;

/// # 本地日志写入器
///
/// 使用tantivy作为日志搜索引擎，支持中文，分词器为jie_ba。
/// 但用jie_ba分词器的性能不太好，看后续能否针对日志场景优化一下。
///
/// 该Writer建议仅用于单机模式
pub struct LocalAppender {
    /// 服务名
    service: String,
    /// 日志通知
    ///
    /// 该通知用于累加计数器
    notify: Arc<Notify>,
    /// 日志存储字段
    fields: Fields,
    /// 索引的writer
    index_writer: Arc<Mutex<IndexWriter>>,
}

/// 日志字段，对应[`LogEntry`]
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

impl LocalAppender {
    const MAX_BUFFER_SIZE: usize = 1000;
    const MEMORY_BUDGET_IN_BYTES: usize = 32 * 1024 * 1024;
    const FLUSH_INTERVAL: Duration = Duration::from_secs(1);
    const TIME_FORMAT: &'static str = "%Y-%m-%d %H:%M:%S%.3f";
    pub fn new<E: Into<String>, S: Into<String>>(dir: E, service: S) -> Self {
        let index = Arc::new(Self::open_or_create_index(&dir.into()).unwrap());

        // 添加jie_ba分词器
        Self::register_tokenizer(&index);

        let schema = index.schema();
        // 日志索引字段
        let fields = Fields::from_schema(&schema);
        // 索引writer
        let index_writer = Arc::new(Mutex::new(
            index.writer(Self::MEMORY_BUDGET_IN_BYTES).unwrap(),
        ));
        let index_writer_clone = index_writer.clone();

        // 计数通知
        let notify = Arc::new(Notify::new());
        let notify_clone = notify.clone();

        let mut interval = tokio::time::interval(Self::FLUSH_INTERVAL);
        let counter = AtomicUsize::new(0);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = notify_clone.notified() => {
                        counter.fetch_add(1, Ordering::SeqCst);
                        if counter.load(Ordering::SeqCst) >= Self::MAX_BUFFER_SIZE {
                            if let Ok(mut writer) = index_writer_clone.lock() {
                                // 提交写入，IO操作
                                let _ = writer.commit();
                            }
                            counter.store(0, Ordering::Relaxed);
                        }
                    }
                    _ = interval.tick() => {
                        if counter.load(Ordering::SeqCst) > 0 {
                            if let Ok(mut writer) = index_writer_clone.lock() {
                                // 提交写入，IO操作
                                let _ = writer.commit();
                            }
                            counter.store(0, Ordering::Relaxed);
                        }
                    }
                }
            }
        });
        Self {
            service: service.into(),
            fields,
            index_writer,
            notify,
        }
    }

    // 打开或创建日志索引
    fn open_or_create_index(dir: &str) -> Result<Index, tantivy::TantivyError> {
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
        Index::open_or_create(MmapDirectory::open(dir)?, schema)
    }

    // 添加jie_ba分词器
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
}

impl Write for LocalAppender {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut log_entry = LogEntry::from(buf.to_vec());
        log_entry.service = self.service.clone();

        let mut doc = TantivyDocument::default();
        doc.add_date(
            self.fields.time,
            Self::string_to_datetime(&log_entry.time).unwrap(),
        );
        doc.add_text(self.fields.service, log_entry.service);
        doc.add_text(self.fields.level, log_entry.level);
        doc.add_text(self.fields.message, log_entry.message);

        // 仅添加，不提交（满足阈值时再写入）
        self.index_writer.lock().unwrap().add_document(doc).unwrap();

        // 通知计数器
        self.notify.notify_one();

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::appender::LocalAppender;
    use crate::{Config, LogAppender, init_log_with};
    use std::env::temp_dir;
    use std::time::Duration;
    use tantivy::collector::{Count, TopDocs};
    use tantivy::query::QueryParser;
    use tantivy::{Document, Index, ReloadPolicy, TantivyDocument};

    #[tokio::test]
    async fn test_log() {
        init_log_with(
            LogAppender::LOCAL,
            Config {
                dir: Some("/tmp/tantivy".to_string()),
                ..Config::default()
            },
        );
        for i in 0..1000000 {
            log::info!("hello world，你好, 今天天气真不错");
            //  println!("i: {}", i);
        }

        tokio::time::sleep(Duration::from_secs(30)).await;
    }

    #[test]
    fn test_query() -> Result<(), Box<dyn std::error::Error>> {
        let index_path = temp_dir().join("tantivy");
        let index = Index::open_in_dir(index_path)?;
        LocalAppender::register_tokenizer(&index);
        let schema = index.schema();
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?;

        let searcher = reader.searcher();

        let query_parser = QueryParser::for_index(
            &index,
            schema.fields().map(|(field, _)| field).collect::<Vec<_>>(),
        );

        //let query = query_parser.parse_query("level:info")?;
        let query = query_parser.parse_query("Hello")?;

        let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;
        for (_score, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;
            println!("{}", retrieved_doc.to_json(&schema));
        }
        let query = query_parser.parse_query("Hello")?;
        let count = searcher.search(&query, &Count)?;
        println!("Found {} documents", count);
        //
        // let query = query_parser.parse_query("title:sea^20 body:whale^70")?;
        //
        // let (_score, doc_address) = searcher
        //     .search(&query, &TopDocs::with_limit(1))?
        //     .into_iter()
        //     .next()
        //     .unwrap();
        //
        // let explanation = query.explain(&searcher, doc_address)?;
        //
        // println!("{}", explanation.to_pretty_json());

        Ok(())
    }
}
