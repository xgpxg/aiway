use protocol::gateway::request_log::RequestLog;
use protocol::logg::{LogSearchReq, LogSearchRes};
use rocket::data::{ByteUnit, FromData, Outcome};
use rocket::serde::json::Json;
use rocket::{Data, Request, State, async_trait, post, routes};
use std::fmt::Debug;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tantivy::aggregation::agg_req::Aggregations;
use tantivy::aggregation::{AggregationCollector, AggregationLimitsGuard};
use tantivy::collector::TopDocs;
use tantivy::directory::MmapDirectory;
use tantivy::query::{Query, QueryParser};
use tantivy::schema::{FAST, Field, INDEXED, STORED, Schema, TEXT, Value};
use tantivy::tokenizer::{LowerCaser, TextAnalyzer};
use tantivy::{
    DateTime, Document, Index, IndexReader, IndexWriter, Order, ReloadPolicy, TantivyDocument,
    TantivyError,
};

struct Fields {
    request_id: Field,
    client_ip: Field,
    client_country: Field,
    client_province: Field,
    client_city: Field,
    method: Field,
    path: Field,
    request_time: Field,
    response_time: Field,
    elapsed: Field,
    status_code: Field,
    response_size: Field,
    user_agent: Field,
    referer: Field,
    node_address: Field,
}

impl Fields {
    fn from_schema(schema: &Schema) -> Self {
        Fields {
            request_id: schema.get_field("request_id").unwrap(),
            client_ip: schema.get_field("client_ip").unwrap(),
            client_country: schema.get_field("client_country").unwrap(),
            client_province: schema.get_field("client_province").unwrap(),
            client_city: schema.get_field("client_city").unwrap(),
            method: schema.get_field("method").unwrap(),
            path: schema.get_field("path").unwrap(),
            request_time: schema.get_field("request_time").unwrap(),
            response_time: schema.get_field("response_time").unwrap(),
            elapsed: schema.get_field("elapsed").unwrap(),
            status_code: schema.get_field("status_code").unwrap(),
            response_size: schema.get_field("response_size").unwrap(),
            user_agent: schema.get_field("user_agent").unwrap(),
            referer: schema.get_field("referer").unwrap(),
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

        // 添加字段，注意不要改变顺序，否则Schema验证会失败
        sb.add_text_field("request_id", TEXT | STORED);
        sb.add_text_field("client_ip", TEXT | STORED | FAST);
        sb.add_text_field("client_country", TEXT | STORED | FAST);
        sb.add_text_field("client_province", TEXT | STORED | FAST);
        sb.add_text_field("client_city", TEXT | STORED | FAST);
        sb.add_text_field("method", STORED);
        sb.add_text_field("path", TEXT | STORED);
        sb.add_date_field("request_time", FAST | STORED);
        sb.add_date_field("response_time", STORED);
        sb.add_i64_field("elapsed", STORED);
        sb.add_u64_field("status_code", INDEXED | STORED);
        sb.add_u64_field("response_size", STORED);
        sb.add_text_field("user_agent", TEXT | STORED);
        sb.add_text_field("referer", TEXT | STORED);
        sb.add_text_field("node_address", TEXT | STORED);

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

    pub fn add(&self, entries: Vec<RequestLog>) {
        let mut index_writer = self.index_writer.lock().unwrap();
        entries.into_iter().for_each(|entry| {
            let mut doc = TantivyDocument::default();
            doc.add_text(self.fields.request_id, &entry.request_id);
            doc.add_text(self.fields.client_ip, &entry.client_ip);

            if let Some(country) = &entry.client_country {
                doc.add_text(self.fields.client_country, country);
            }

            if let Some(province) = &entry.client_province {
                doc.add_text(self.fields.client_province, province);
            }

            if let Some(city) = &entry.client_city {
                doc.add_text(self.fields.client_city, city);
            }

            doc.add_text(self.fields.method, &entry.method);
            doc.add_text(self.fields.path, &entry.path);

            doc.add_date(
                self.fields.request_time,
                DateTime::from_timestamp_millis(entry.request_time),
            );

            doc.add_date(
                self.fields.response_time,
                DateTime::from_timestamp_millis(entry.response_time),
            );

            doc.add_i64(self.fields.elapsed, entry.elapsed);
            doc.add_u64(self.fields.status_code, entry.status_code as u64);

            if let Some(size) = entry.response_size {
                doc.add_u64(self.fields.response_size, size as u64);
            }

            if let Some(ua) = &entry.user_agent {
                doc.add_text(self.fields.user_agent, ua);
            }

            if let Some(referer) = &entry.referer {
                doc.add_text(self.fields.referer, referer);
            }
            doc.add_text(self.fields.node_address, &entry.node_address);

            let _ = index_writer.add_document(doc);
        });
        index_writer.commit().unwrap();
    }

    pub fn search(&self, req: LogSearchReq) -> anyhow::Result<LogSearchRes<RequestLog>> {
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
                DateTime::from_timestamp_secs(start_timestamp /* + Self::TIME_OFFSET*/)
            ));
        }

        if let Some(end_timestamp) = req.end_timestamp {
            query.push(format!(
                "request_time:<{:?}",
                DateTime::from_timestamp_secs(end_timestamp /* + Self::TIME_OFFSET*/)
            ));
        }

        if query.is_empty() {
            query.push("*".to_string());
        }

        //println!("query: {}", query.join(" AND "));

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
            let mut log_entry = RequestLog::default();
            for (field, value) in retrieved_doc.iter_fields_and_values() {
                match field.field_id() {
                    fid if fid == self.fields.request_time.field_id() => {
                        log_entry.request_time =
                            value.as_datetime().unwrap().into_timestamp_millis();
                    }
                    fid if fid == self.fields.response_time.field_id() => {
                        log_entry.response_time =
                            value.as_datetime().unwrap().into_timestamp_millis();
                    }
                    fid if fid == self.fields.request_id.field_id() => {
                        log_entry.request_id =
                            value.as_str().map(|s| s.to_string()).unwrap_or_default();
                    }
                    fid if fid == self.fields.client_ip.field_id() => {
                        log_entry.client_ip =
                            value.as_str().map(|s| s.to_string()).unwrap_or_default();
                    }
                    fid if fid == self.fields.client_country.field_id() => {
                        log_entry.client_country = value.as_str().map(|s| s.to_string());
                    }
                    fid if fid == self.fields.client_province.field_id() => {
                        log_entry.client_province = value.as_str().map(|s| s.to_string());
                    }
                    fid if fid == self.fields.client_city.field_id() => {
                        log_entry.client_city = value.as_str().map(|s| s.to_string());
                    }
                    fid if fid == self.fields.method.field_id() => {
                        log_entry.method =
                            value.as_str().map(|s| s.to_string()).unwrap_or_default();
                    }
                    fid if fid == self.fields.path.field_id() => {
                        log_entry.path = value.as_str().map(|s| s.to_string()).unwrap_or_default();
                    }
                    fid if fid == self.fields.status_code.field_id() => {
                        log_entry.status_code =
                            value.as_u64().map(|v| v as u16).unwrap_or_default();
                    }
                    fid if fid == self.fields.elapsed.field_id() => {
                        log_entry.elapsed = value.as_i64().unwrap_or_default();
                    }
                    fid if fid == self.fields.response_size.field_id() => {
                        log_entry.response_size = value.as_u64().map(|v| v as usize);
                    }
                    fid if fid == self.fields.user_agent.field_id() => {
                        log_entry.user_agent = value.as_str().map(|s| s.to_string());
                    }
                    fid if fid == self.fields.referer.field_id() => {
                        log_entry.referer = value.as_str().map(|s| s.to_string());
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
}

#[derive(Debug)]
struct LogEntries(Vec<RequestLog>);

#[async_trait]
impl<'r> FromData<'r> for LogEntries {
    type Error = ();

    async fn from_data(_req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
        let bytes = data.open(ByteUnit::Mebibyte(8)).into_bytes().await.unwrap();
        let lines = String::from_utf8(bytes.value).unwrap();
        let lines = lines.lines();

        let entries = lines
            .map(|line| serde_json::from_str(line).unwrap())
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
fn search(req: Json<LogSearchReq>, logg: &State<Logg>) -> Json<LogSearchRes<RequestLog>> {
    match logg.search(req.0) {
        Ok(res) => Json(res),
        Err(e) => {
            println!("Error: {}", e);
            Json(LogSearchRes::default())
        }
    }
}
