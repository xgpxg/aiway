//! # 插件
//! ## 基本准则
//! - 插件是网关实现业务逻辑的核心组件，应提供良好的设计，且不应该频繁变更定义。
//! - 插件与网关之间通过可序列化的数据通信。
//! - 插件模块仅提供插件定义、加载、及执行，不应提供插件注册、卸载等管理功能，这些功能交给网关自身实现。
//!
//! ## 插件分类
//! ### 全局插件
//! 全局插件对整个网关的所有请求生效（不含控制台请求，因为控制台是独立的）。
//!
//! 全局插件分两个阶段执行：
//! - 请求阶段：在请求到达API处理端点前执行，可对请求改写、安全验证、限流、缓存等。
//! - 响应阶段：在API处理完成，响应客户端前执行，可修改响应、记录日志等。该阶段的插件可通过参数控制当前一个插件中断时，是否仍然执行。
//!
//! 各阶段的插件按配置的顺序依次执行，可被中断。
//!
//! 中断处理：
//! - 请求阶段中断：转发到特殊的API端点，执行响应，此时，后置阶段的插件仍会执行
//! - 响应阶段中断：返回错误响应。由于日志拦截是在最后一步执行，所以，返回错误响应后，日志仍然能被记录。
//!
//! ### 路由插件
//! 对特定路由生效。
//!
//! 路由插件和全局插件实现方式相同，仅执行时机不同。
//!
//! ## 使用方式
//! ```rust
//! // 示例插件
//! pub struct DemoPlugin;
//!
//! impl DemoPlugin {
//!     pub fn new() -> Self {
//!         Self {}
//!     }
//! }
//! #[async_trait]
//! impl Plugin for DemoPlugin {
//!     fn name(&self) -> &'static str {
//!         "demo"
//!     }
//!
//!     // 实现插件逻辑
//!     async fn execute(&self, context: &HttpContext, config: &Value) -> Result<(), PluginError> {
//!         println!("run demo plugin, context: {:?}", context);
//!         Ok(())
//!     }
//! }
//!
//! // 导出插件
//! export!(DemoPlugin);
//! ```
//!

mod macros;
mod manager;
mod network;

use crate::network::NETWORK;
pub use async_trait::async_trait;
use libloading::Symbol;
pub use manager::PluginManager;
use protocol::gateway::HttpContext;
pub use semver::Version;
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

/// 插件定义
///
/// - name
///
/// 插件的名称，原则上不要重复。在`PluginManager`中，如果重复了，后添加的将被覆盖。
///
/// - execute
///
/// `execute`接收HttpContext参数，该HttpContext是可变的（内部可变性），可在插件逻辑内部修改请求和响应。
/// 注意：当多个插件修改HttpContext的同一个属性时，后执行的插件会覆盖前一个插件的修改。
/// 插件实现方应该自行决定插件运行阶段（请求阶段或者响应阶段），从而获取或修改request或response的数据。
///
/// - 返回值
/// 返回[serde_json:Value]
///
#[async_trait]
pub trait Plugin: Send + Sync {
    /// 插件名称
    fn name(&self) -> &str;
    /// 插件信息
    fn info(&self) -> PluginInfo;
    /// 执行插件
    async fn execute(&self, context: &HttpContext, config: &Value) -> Result<Value, PluginError>;
}

/// 插件信息
#[derive(Debug)]
pub struct PluginInfo {
    /// 插件类型
    pub plugin_type: PluginType,
    /// 插件版本
    pub version: Version,
    /// 默认配置
    pub default_config: Value,
}

/// 插件类型
#[derive(Debug)]
pub enum PluginType {
    /// 网关插件
    Gateway,
    /// AI插件
    AI,
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

    async fn execute(&self, context: &HttpContext, config: &Value) -> Result<Value, PluginError> {
        self.plugin.execute(context, config).await
    }
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
        let temp = temp_dir().join(format!("{}.so", uuid::Uuid::new_v4().to_string()));
        fs::write(&temp, from).map_err(|e| PluginError::LoadError(e.to_string()))?;
        temp.try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manager::PluginManager;
    use std::io::Read;
    #[tokio::test]
    async fn test_network_plugin() {
        let p = NetworkPlugin(
            "http://192.168.1.242:10000/aiway/test/plugins/libdemo_plugin.so".to_string(),
        );
        let plugin: Box<dyn Plugin> = p.async_try_into().await.unwrap();
        plugin
            .execute(&HttpContext::default(), &Value::Null)
            .await
            .unwrap();
    }
    #[tokio::test]
    async fn test_plugin_manager() {
        let p = NetworkPlugin(
            "http://192.168.1.242:10000/aiway/test/plugins/libdemo_plugin.so".to_string(),
        );
        let plugin: Box<dyn Plugin> = p.async_try_into().await.unwrap();
        let mut manager = PluginManager::new();
        manager.register(plugin);
        manager
            .run("demo", &HttpContext::default(), &Value::Null)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_plugin_from_bytes() {
        let file =
            File::open("../../target/release/libaha_model_request_wrapper_plugin.so").unwrap();
        // 获取file的bytes
        let bytes = file.bytes().collect::<Result<Vec<_>, _>>().unwrap();
        let plugin: Box<dyn Plugin> = bytes.try_into().unwrap();
        println!("{:?}", plugin.info());
    }
}
