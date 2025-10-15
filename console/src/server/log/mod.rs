
pub mod api;
mod request;
mod response;
mod service;

pub trait LogSearcher {
    fn search(&self, options: LogSearchOptions);
}

pub struct LogSearchOptions {
    pub info: Option<String>,
    pub service: Option<String>,
    pub level: Option<String>,
    pub message: Option<String>,
    pub time_range: Option<(Option<String>, Option<String>)>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

//#[cfg(feature = "cluster")]
struct QuickwitSearcher;

//#[cfg(feature = "cluster")]
impl LogSearcher for QuickwitSearcher {
    fn search(&self, options: LogSearchOptions) {
        todo!()
    }
}

#[cfg(feature = "standalone")]
struct LocalSearcher;

#[cfg(feature = "standalone")]
impl LogSearcher for LocalSearcher {
    fn search(&self, options: LogSearchOptions) {
        use logging::tantivy;

        todo!()
    }
}
