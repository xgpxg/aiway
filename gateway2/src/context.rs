use aiway_protocol::gateway::ConfiguredPlugin;
use dashmap::DashMap;
use std::sync::LazyLock;

pub struct RequestContext {
    inner: DashMap<String, State>,
}
pub struct State {
    pub plugins: Option<Vec<ConfiguredPlugin>>,
}

pub fn init() {
    let _ = &*GLOBAL_STATE;
}

pub static GLOBAL_STATE: LazyLock<RequestContext> = LazyLock::new(RequestContext::new);

impl RequestContext {
    pub fn new() -> Self {
        RequestContext {
            inner: Default::default(),
        }
    }

    pub fn append_plugin(request_id: String, plugin: ConfiguredPlugin) {
        GLOBAL_STATE
            .inner
            .entry(request_id)
            .or_insert_with(|| State {
                plugins: Some(vec![plugin]),
            });
    }
}
