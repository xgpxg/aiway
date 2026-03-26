#[doc = include_str!("../README.md")]
mod macros;
mod network;

use crate::network::NETWORK;
// #[cfg(feature = "model")]
// pub use aiway_model_protocol as model_protocol;
pub use aiway_protocol as protocol;
use aiway_protocol::context::http::{request, response};
pub use async_trait::async_trait;
pub use bytes::Bytes;
use libloading::Symbol;
pub use log;
use protocol::context::HttpContext;
pub use semver::Version;
use serde::{Deserialize, Serialize};
pub use serde_json;
use serde_json::Value;
use std::env::temp_dir;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
#[derive(Debug)]
pub enum PluginError {
    /// 执行插件业务逻辑时的错误
    ExecuteError(String),
    /// 插件不存在
    NotFound(String),
    /// 从磁盘或网络加载插件时错误
    LoadError(String),
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginError::ExecuteError(msg) => write!(f, "{}", msg),
            PluginError::NotFound(msg) => write!(f, "{}", msg),
            PluginError::LoadError(msg) => write!(f, "{}", msg),
        }
    }
}

#[async_trait]
pub trait Plugin: Send + Sync {
    /// 插件名称
    fn name(&self) -> &str;
    /// 插件信息
    fn info(&self) -> PluginInfo;

    /// 请求阶段，可改写头部
    async fn on_request(
        &self,
        _config: &Value,
        _head: &mut request::Parts,
        _ctx: &mut HttpContext,
    ) -> Result<(), PluginError> {
        Ok(())
    }

    /// 请求体阶段，可改写请求体
    async fn on_request_body(
        &self,
        _config: &Value,
        _body: &mut Option<Bytes>,
        _ctx: &mut HttpContext,
    ) -> Result<(), PluginError> {
        Ok(())
    }

    /// 响应阶段，可改写头部
    async fn on_response(
        &self,
        _config: &Value,
        _head: &mut response::Parts,
        _ctx: &mut HttpContext,
    ) -> Result<(), PluginError> {
        Ok(())
    }

    /// 响应体阶段，可改写响应体
    fn on_response_body(
        &self,
        _config: &Value,
        _body: &mut Option<Bytes>,
        _ctx: &mut HttpContext,
    ) -> Result<(), PluginError> {
        Ok(())
    }

    async fn on_logging(&self, _: &Value, _: &mut HttpContext) {}
}

/// 插件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// 插件版本
    pub version: Version,
    /// 默认配置
    pub default_config: Value,
    /// 描述
    pub description: String,
}

impl TryFrom<PathBuf> for Box<dyn Plugin> {
    type Error = PluginError;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        unsafe {
            let lib = libloading::Library::new(&value)
                .map_err(|e| PluginError::LoadError(e.to_string()))?;

            let create_plugin: Symbol<unsafe extern "C" fn() -> *mut dyn Plugin> = lib
                .get(b"create_plugin")
                .map_err(|e| PluginError::LoadError(e.to_string()))?;

            let plugin_ptr = create_plugin();

            if plugin_ptr.is_null() {
                return Err(PluginError::LoadError(
                    "Failed to create plugin: ptr is null".to_string(),
                ));
            }

            let plugin = Box::from_raw(plugin_ptr);

            // 包装一层，保持对lib的引用
            let wrapped_plugin = Box::new(LibraryPluginWrapper { plugin, _lib: lib });

            Ok(wrapped_plugin)
        }
    }
}

struct LibraryPluginWrapper {
    plugin: Box<dyn Plugin>,
    _lib: libloading::Library,
}

#[async_trait]
impl Plugin for LibraryPluginWrapper {
    fn name(&self) -> &str {
        self.plugin.name()
    }

    fn info(&self) -> PluginInfo {
        self.plugin.info()
    }

    async fn on_request(
        &self,
        config: &Value,
        head: &mut request::Parts,
        ctx: &mut HttpContext,
    ) -> Result<(), PluginError> {
        self.plugin.on_request(config, head, ctx).await
    }

    async fn on_request_body(
        &self,
        config: &Value,
        body: &mut Option<Bytes>,
        ctx: &mut HttpContext,
    ) -> Result<(), PluginError> {
        self.plugin.on_request_body(config, body, ctx).await
    }
    async fn on_response(
        &self,
        config: &Value,
        head: &mut response::Parts,
        ctx: &mut HttpContext,
    ) -> Result<(), PluginError> {
        self.plugin.on_response(config, head, ctx).await
    }
    fn on_response_body(
        &self,
        config: &Value,
        body: &mut Option<Bytes>,
        ctx: &mut HttpContext,
    ) -> Result<(), PluginError> {
        self.plugin.on_response_body(config, body, ctx)
    }

    async fn on_logging(&self, _: &Value, _: &mut HttpContext) {}

}

impl Drop for LibraryPluginWrapper {
    fn drop(&mut self) {
        unsafe {
            let destructor: Symbol<unsafe extern "C" fn(*mut dyn Plugin)> = self
                ._lib
                .get(b"destroy_plugin")
                .expect("Failed to get destructor function");

            destructor(self.plugin.as_mut());
        }
    }
}

/// 从指定的URL加载插件
pub struct NetworkPlugin(pub String);

#[async_trait]
pub trait AsyncTryInto<T>: Sized {
    type Error;

    async fn async_try_into(self) -> Result<T, Self::Error>;
}

#[async_trait]
impl AsyncTryInto<Box<dyn Plugin>> for NetworkPlugin {
    type Error = PluginError;

    async fn async_try_into(self) -> Result<Box<dyn Plugin>, Self::Error> {
        let response = NETWORK
            .client
            .get(&self.0)
            .send()
            .await
            .map_err(|e| PluginError::LoadError(e.to_string()))?
            .error_for_status()
            .map_err(|e| PluginError::LoadError(e.to_string()))?;

        let bytes = response
            .bytes()
            .await
            .map_err(|e| PluginError::LoadError(e.to_string()))?;

        let tpf = temp_dir().join(uuid::Uuid::new_v4().to_string());

        let plugin = {
            let tpf = tpf.clone();
            let mut file = File::create(&tpf).map_err(|e| PluginError::LoadError(e.to_string()))?;

            file.write_all(&bytes)
                .map_err(|e| PluginError::LoadError(e.to_string()))?;

            drop(file);

            tpf.try_into()
        };

        fs::remove_file(tpf).map_err(|e| PluginError::LoadError(e.to_string()))?;

        plugin
    }
}

impl TryFrom<Vec<u8>> for Box<dyn Plugin> {
    type Error = PluginError;

    fn try_from(from: Vec<u8>) -> Result<Box<dyn Plugin>, Self::Error> {
        let temp = temp_dir().join(format!("{}.so", uuid::Uuid::new_v4()));
        fs::write(&temp, from).map_err(|e| PluginError::LoadError(e.to_string()))?;
        temp.try_into()
    }
}
