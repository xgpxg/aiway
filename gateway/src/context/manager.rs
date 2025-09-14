use crate::context::Headers;
use dashmap::DashMap;
use protocol::gateway::RequestContext;
use rocket::Request;
use std::sync::{Arc, LazyLock};

/// 请求上下文管理器
pub struct RequestContextManager {
    contexts: DashMap<String, Arc<RequestContext>>,
}

impl RequestContextManager {
    pub fn new() -> Self {
        Self {
            contexts: DashMap::new(),
        }
    }

    pub fn get(&self, id: &str) -> Arc<RequestContext> {
        match self.contexts.get(id) {
            Some(v) => v.value().clone(),
            None => {
                panic!("context not found in current request");
            }
        }
    }

    pub fn get_from_request(&self, req: &Request) -> Arc<RequestContext> {
        let id = req.headers().get_one(Headers::REQUEST_ID).unwrap();
        RCM.get(id)
    }

    pub fn set(&self, id: &str, context: Arc<RequestContext>) {
        self.contexts.insert(id.to_string(), context);
    }

    pub fn remove(&self, id: &str) {
        self.contexts.remove(id);
    }
}

pub static RCM: LazyLock<RequestContextManager> = LazyLock::new(|| RequestContextManager::new());
