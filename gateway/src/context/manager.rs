use crate::context::Headers;
use anyhow::Context;
use dashmap::{DashMap, DashSet};
use protocol::gateway::{HttpContext, RequestContext};
use rocket::data::ToByteUnit;
use rocket::{Data, Request};
use std::sync::{Arc, LazyLock};
use uuid::Uuid;

/// HTTP上下文管理器
pub struct HttpContextManager {
    contexts: DashMap<String, Arc<HttpContext>>,
}

pub static HCM: LazyLock<HttpContextManager> = LazyLock::new(|| HttpContextManager::new());

impl HttpContextManager {
    pub fn new() -> Self {
        Self {
            contexts: DashMap::new(),
        }
    }

    /// 通过请求ID获取上下文
    pub fn get(&self, id: &str) -> Arc<HttpContext> {
        match self.contexts.get(id) {
            Some(v) => v.value().clone(),
            None => {
                panic!("context not found in current request");
            }
        }
    }

    /// 从rocket的request中获取上下文
    pub fn get_from_request(&self, req: &Request) -> Arc<HttpContext> {
        let id = req
            .headers()
            .get_one(Headers::REQUEST_ID)
            .context("request id not found, maybe not call request fairing")
            .unwrap();
        HCM.get(id)
    }

    /// 设置上下文
    ///
    /// 相同请求ID的会覆盖
    pub fn set(&self, id: &str, context: Arc<HttpContext>) {
        self.contexts.insert(id.to_string(), context);
    }

    /// 移除上下文
    pub fn remove(&self, id: &str) {
        self.contexts.remove(id);
    }
}
