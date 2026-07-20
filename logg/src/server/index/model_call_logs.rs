use aiway_protocol::gateway::ModelCallLog;
use aiway_protocol::logg::{LogDeleteReq, LogSearchReq, LogSearchRes};
use rocket::data::{ByteUnit, FromData, Outcome};
use rocket::serde::json::Json;
use rocket::{Data, Request, State, async_trait, post, routes};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tantivy::aggregation::agg_req::Aggregations;
use tantivy::aggregation::{AggregationCollector, AggregationLimitsGuard};
use tantivy::collector::TopDocs;
use tantivy::directory::MmapDirectory;
use tantivy::query::{Query, QueryParser};
use tantivy::schema::{DateOptions, DateTimePrecision, FAST, Field, STORED, Schema, TEXT, Value};
use tantivy::tokenizer::{LowerCaser, TextAnalyzer};
use tantivy::{
    DateTime, Document, Index, IndexReader, IndexWriter, Order, ReloadPolicy, TantivyDocument,
    TantivyError,
};

struct Fields {
    request_id: Field,
    model_name: Field,
    provider_name: Field,
    request_time: Field,
    response_time: Field,
    elapsed: Field,
    ttft_ms: Field,
    status_code: Field,
    is_stream: Field,
    prompt_tokens: Field,
    completion_tokens: Field,
    total_tokens: Field,
    node_address: Field,
}

impl Fields {
    fn from_schema(schema: &Schema) -> Self {
        Fields {
            request_id: schema.get_field("request_id").unwrap(),
            model_name: schema.get_field("model_name").unwrap(),
            provider_name: schema.get_field("provider_name").unwrap(),
            request_time: schema.get_field("request_time").unwrap(),
            response_time: schema.get_field("response_time").unwrap(),
            elapsed: schema.get_field("elapsed").unwrap(),
            ttft_ms: schema.get_field("ttft_ms").unwrap(),
            status_code: schema.get_field("status_code").unwrap(),
            is_stream: schema.get_field("is_stream").unwrap(),
            prompt_tokens: schema.get_field("prompt_tokens").unwrap(),
            completion_tokens: schema.get_field("completion_tokens").unwrap(),
            total_tokens: schema.get_field("total_tokens").unwrap(),
            node_address: schema.get_field("node_address").unwrap(),
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

    pub(crate) fn new(dir: &str) -> Result<Self, TantivyError> {
        let index = Self::open_or_create_index(dir)?;
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

        sb.add_text_field("request_id", TEXT | STORED | FAST);
        sb.add_text_field("model_name", TEXT | STORED | FAST);
        sb.add_text_field("provider_name", TEXT | STORED | FAST);
        sb.add_date_field(
            "request_time",
            DateOptions::default()
                .set_fast()
                .set_precision(DateTimePrecision::Microseconds)
                | STORED,
        );
        sb.add_date_field("response_time", STORED);
        sb.add_i64_field("elapsed", FAST | STORED);
        sb.add_i64_field("ttft_ms", FAST | STORED);
        sb.add_u64_field("status_code", FAST | STORED);
        sb.add_bool_field("is_stream", FAST | STORED);
        sb.add_i64_field("prompt_tokens", FAST | STORED);
        sb.add_i64_field("completion_tokens", FAST | STORED);
        sb.add_i64_field("total_tokens", FAST | STORED);
        sb.add_text_field("node_address", TEXT | STORED);

        let schema = sb.build();

        if !Path::new(dir).exists() {
            fs::create_dir_all(dir)?;
        }

        Index::open_or_create(MmapDirectory::open(dir)?, schema)
    }

    fn register_tokenizer(index: &Index) {
        let tokenizer = tantivy_jieba::JiebaTokenizer::new();
        let analyzer = TextAnalyzer::builder(tokenizer)
            .filter(LowerCaser)
            .build();
        index.tokenizers().register("jie_ba", analyzer);
    }

    pub fn add(&self, entries: Vec<ModelCallLog>) {
        let mut index_writer = self.index_writer.lock().unwrap();
        entries.into_iter().for_each(|entry| {
            let mut doc = TantivyDocument::default();
            doc.add_text(self.fields.request_id, &entry.request_id);
            doc.add_text(self.fields.model_name, &entry.model_name);
            doc.add_text(self.fields.provider_name, &entry.provider_name);
            doc.add_date(
                self.fields.request_time,
                DateTime::from_timestamp_millis(entry.request_time),
            );
            doc.add_date(
                self.fields.response_time,
                DateTime::from_timestamp_millis(entry.response_time),
            );
            doc.add_i64(self.fields.elapsed, entry.elapsed);
            if let Some(ttft) = entry.ttft_ms {
                doc.add_i64(self.fields.ttft_ms, ttft);
            }
            doc.add_u64(self.fields.status_code, entry.status_code as u64);
            doc.add_bool(self.fields.is_stream, entry.is_stream);
            if let Some(v) = entry.prompt_tokens {
                doc.add_i64(self.fields.prompt_tokens, v);
            }
            if let Some(v) = entry.completion_tokens {
                doc.add_i64(self.fields.completion_tokens, v);
            }
            if let Some(v) = entry.total_tokens {
                doc.add_i64(self.fields.total_tokens, v);
            }
            doc.add_text(self.fields.node_address, &entry.node_address);

            let _ = index_writer.add_document(doc);
        });
        index_writer.commit().unwrap();
    }

    pub fn search(&self, req: LogSearchReq) -> anyhow::Result<LogSearchRes<ModelCallLog>> {
        let schema = self.index.schema();
        let query_parser =
            QueryParser::for_index(&self.index, schema.fields().map(|(f, _)| f).collect());

        let mut query = vec![];

        if let Some(q) = req.query
            && !q.is_empty()
        {
            query.push(q);
        }

        if let Some(start_timestamp) = req.start_timestamp {
            query.push(format!(
                "request_time:>={:?}",
                DateTime::from_timestamp_secs(start_timestamp)
            ));
        }

        if let Some(end_timestamp) = req.end_timestamp {
            query.push(format!(
                "request_time:<{:?}",
                DateTime::from_timestamp_secs(end_timestamp)
            ));
        }

        if query.is_empty() {
            query.push("*".to_string());
        }

        let query = query_parser.parse_query(&query.join(" AND "))?;
        let searcher = self.reader.searcher();
        let num_hits = query.count(&searcher)?;
        if num_hits == 0 {
            return Ok(LogSearchRes::default());
        }

        let agg = if let Some(agg) = req.aggs {
            match serde_json::from_value::<Aggregations>(agg) {
                Ok(aggregations) => Some(AggregationCollector::from_aggs(
                    aggregations,
                    AggregationLimitsGuard::default(),
                )),
                Err(e) => {
                    eprintln!("Failed to parse aggregations: {}", e);
                    None
                }
            }
        } else {
            None
        };

        let (top_docs, agg_result): (Vec<(DateTime, _)>, _) = searcher.search(
            &query,
            &(
                TopDocs::with_limit(req.max_hits)
                    .and_offset(req.start_offset)
                    .order_by_fast_field("request_time", Order::Desc),
                agg,
            ),
        )?;
        let agg_json = serde_json::to_value(&agg_result)?;

        let mut list = Vec::new();
        for (_score, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;
            let mut log_entry = ModelCallLog {
                request_id: String::new(),
                model_name: String::new(),
                provider_name: String::new(),
                request_time: 0,
                response_time: 0,
                elapsed: 0,
                ttft_ms: None,
                status_code: 0,
                is_stream: false,
                prompt_tokens: None,
                completion_tokens: None,
                total_tokens: None,
                node_address: String::new(),
            };
            for (field, value) in retrieved_doc.iter_fields_and_values() {
                match field.field_id() {
                    fid if fid == self.fields.request_id.field_id() => {
                        log_entry.request_id =
                            value.as_str().map(|s| s.to_string()).unwrap_or_default();
                    }
                    fid if fid == self.fields.model_name.field_id() => {
                        log_entry.model_name =
                            value.as_str().map(|s| s.to_string()).unwrap_or_default();
                    }
                    fid if fid == self.fields.provider_name.field_id() => {
                        log_entry.provider_name =
                            value.as_str().map(|s| s.to_string()).unwrap_or_default();
                    }
                    fid if fid == self.fields.request_time.field_id() => {
                        log_entry.request_time =
                            value.as_datetime().unwrap().into_timestamp_millis();
                    }
                    fid if fid == self.fields.response_time.field_id() => {
                        log_entry.response_time =
                            value.as_datetime().unwrap().into_timestamp_millis();
                    }
                    fid if fid == self.fields.elapsed.field_id() => {
                        log_entry.elapsed = value.as_i64().unwrap_or_default();
                    }
                    fid if fid == self.fields.ttft_ms.field_id() => {
                        log_entry.ttft_ms = value.as_i64();
                    }
                    fid if fid == self.fields.status_code.field_id() => {
                        log_entry.status_code = value.as_u64().unwrap_or(0) as u16;
                    }
                    fid if fid == self.fields.is_stream.field_id() => {
                        log_entry.is_stream = value.as_bool().unwrap_or(false);
                    }
                    fid if fid == self.fields.prompt_tokens.field_id() => {
                        log_entry.prompt_tokens = value.as_i64();
                    }
                    fid if fid == self.fields.completion_tokens.field_id() => {
                        log_entry.completion_tokens = value.as_i64();
                    }
                    fid if fid == self.fields.total_tokens.field_id() => {
                        log_entry.total_tokens = value.as_i64();
                    }
                    fid if fid == self.fields.node_address.field_id() => {
                        log_entry.node_address =
                            value.as_str().map(|s| s.to_string()).unwrap_or_default();
                    }
                    _ => {}
                }
            }
            list.push(log_entry);
        }

        Ok(LogSearchRes {
            num_hits,
            hits: list,
            aggregations: Some(agg_json),
        })
    }

    pub fn delete(&self, req: LogDeleteReq) -> anyhow::Result<()> {
        let schema = self.index.schema();
        let query_parser =
            QueryParser::for_index(&self.index, schema.fields().map(|(f, _)| f).collect());

        let mut query = vec![];

        if let Some(start_timestamp) = req.start_timestamp {
            query.push(format!(
                "request_time:>={:?}",
                DateTime::from_timestamp_secs(start_timestamp)
            ));
        }

        if let Some(end_timestamp) = req.end_timestamp {
            query.push(format!(
                "request_time:<{:?}",
                DateTime::from_timestamp_secs(end_timestamp)
            ));
        }

        if query.is_empty() {
            query.push("*".to_string());
        }

        let query = query_parser.parse_query(&query.join(" AND "))?;
        let mut index_writer = self.index_writer.lock().unwrap();
        let _ = index_writer.delete_query(query);
        index_writer.commit()?;

        Ok(())
    }
}

#[derive(Debug)]
struct LogEntries(Vec<ModelCallLog>);

#[async_trait]
impl<'r> FromData<'r> for LogEntries {
    type Error = ();

    async fn from_data(_req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
        let bytes = data.open(ByteUnit::Mebibyte(8)).into_bytes().await.unwrap();
        let lines = String::from_utf8(bytes.value).unwrap();
        let entries = lines
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect::<Vec<_>>();

        Outcome::Success(LogEntries(entries))
    }
}

pub fn routes() -> Vec<rocket::Route> {
    routes![ingest, search, delete]
}

#[post("/ingest", data = "<req>")]
fn ingest(req: LogEntries, logg: &State<Logg>) {
    logg.add(req.0);
}

#[post("/search", data = "<req>")]
fn search(req: Json<LogSearchReq>, logg: &State<Logg>) -> Json<LogSearchRes<ModelCallLog>> {
    match logg.search(req.0) {
        Ok(res) => Json(res),
        Err(e) => {
            println!("Error: {}", e);
            Json(LogSearchRes::default())
        }
    }
}

#[post("/delete", data = "<req>")]
fn delete(req: Json<LogDeleteReq>, logg: &State<Logg>) {
    match logg.delete(req.0) {
        Ok(_) => (),
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
