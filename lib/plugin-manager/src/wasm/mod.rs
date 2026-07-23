//! WASM 插件运行环境实现。
//!
mod host_functions;
mod network;

use self::network::NETWORK;
use aiway_plugin::PluginContext;
use aiway_plugin::wasm_types::{
    HOOK_ON_LOGGING, HOOK_ON_REQUEST, HOOK_ON_REQUEST_BODY, HOOK_ON_RESPONSE,
    HOOK_ON_RESPONSE_BODY, WasmHead, WasmInput, WasmOutput, WasmPluginInfo,
};
pub use aiway_plugin::{Plugin, PluginError, PluginInfo};
pub use aiway_protocol as protocol;
use aiway_protocol::context::HttpContext;
use aiway_protocol::context::http::{request, response};
pub use async_trait::async_trait;
pub use bytes::Bytes;
use crossbeam::queue::ArrayQueue;
pub use http;
pub use semver::Version;
pub use serde_json;
use serde_json::Value;
use std::cell::UnsafeCell;
use std::fs;
use std::ptr::NonNull;
use std::sync::Arc;
use std::sync::{LazyLock, OnceLock};
use wasmtime::{
    Engine, Instance, InstanceAllocationStrategy, Linker, Memory, Module, PoolingAllocationConfig,
    Store, TypedFunc,
};

/// 全局共享 Engine
static ENGINE: LazyLock<Engine> = LazyLock::new(|| {
    let mut config = wasmtime::Config::new();

    let mut pooling = PoolingAllocationConfig::new();
    pooling
        .total_core_instances(1024)
        .total_memories(1024)
        .total_tables(1024)
        .max_memory_size(2 * 1024 * 1024); // 单实例最大 2MB

    config.allocation_strategy(InstanceAllocationStrategy::Pooling(pooling));

    Engine::new(&config).expect("failed to create wasmtime Engine with pooling allocator")
});

/// WASM Store 上下文
pub(crate) struct WasmStoreCtx {
    //pub(crate) wasi: WasiP1Ctx,
    /// 当前请求的 HttpContext 指针，每次 call_wasm 前设置，调用后清除
    pub(crate) http_ctx: UnsafeCell<Option<NonNull<HttpContext>>>,
    /// 当前插件名称指针，call_wasm 期间有效，WasmPlugin 生命周期内不变
    pub(crate) plugin_name: UnsafeCell<Option<NonNull<str>>>,
}

impl WasmStoreCtx {
    fn new() -> Self {
        Self {
            http_ctx: UnsafeCell::new(None),
            plugin_name: UnsafeCell::new(None),
        }
    }
}

// SAFETY: http_ctx 指针仅在 call_wasm 期间设置，
// 且当前线程被阻塞或独占，不存在跨线程并发访问。
unsafe impl Send for WasmStoreCtx {}

/// 创建异步 Linker。
/// 同步的会导致嵌套的异步运行时，会报错。
fn create_linker() -> Result<Linker<WasmStoreCtx>, PluginError> {
    let mut linker = Linker::new(&ENGINE);
    host_functions::register(&mut linker)?;
    Ok(linker)
}

/// 创建 Store（运行时使用）
fn create_store() -> Store<WasmStoreCtx> {
    Store::new(&ENGINE, WasmStoreCtx::new())
}

/// 创建加载插件信息用的 Linker
fn create_load_linker(store: &mut Store<()>, module: &Module) -> Result<Linker<()>, PluginError> {
    let mut linker = Linker::new(&ENGINE);
    linker
        .define_unknown_imports_as_default_values(store, module)
        .map_err(|e| PluginError::LoadError(format!("stub unknown imports failed: {}", e)))?;
    Ok(linker)
}

/// 预解析的 WASM 导出函数句柄
///
/// `get_typed_func` 内部做两次哈希表查找（导出名 + 类型签名），
/// 在高频调用下开销显著。这些句柄在 Instance 生命周期内不变，
/// 一次性解析后复用，将查找开销从 O(n) 降至 O(1)。
#[derive(Clone)]
struct CachedFunc {
    memory: Memory,
    alloc_fn: TypedFunc<i32, i32>,
    call_fn: TypedFunc<(i32, i32, i32), i64>,
    dealloc_fn: Option<TypedFunc<(i32, i32), ()>>,
}

impl CachedFunc {
    /// 从 Instance 一次性解析所有导出函数
    fn resolve(store: &mut Store<WasmStoreCtx>, instance: &Instance) -> Result<Self, PluginError> {
        let memory = instance
            .get_memory(&mut *store, "memory")
            .ok_or_else(|| PluginError::ExecuteError("no 'memory' export".into()))?;
        let alloc_fn = instance
            .get_typed_func::<i32, i32>(&mut *store, "aiway_alloc")
            .map_err(|e| PluginError::ExecuteError(format!("get aiway_alloc failed: {}", e)))?;
        let call_fn = instance
            .get_typed_func::<(i32, i32, i32), i64>(&mut *store, "aiway_call")
            .map_err(|e| PluginError::ExecuteError(format!("get aiway_call failed: {}", e)))?;
        let dealloc_fn = instance
            .get_typed_func::<(i32, i32), ()>(&mut *store, "aiway_dealloc")
            .ok();
        Ok(Self {
            memory,
            alloc_fn,
            call_fn,
            dealloc_fn,
        })
    }
}

/// 缓存已创建的实例。
/// 避免每次 Hook 调用都重新实例化或重新解析导出函数。
/// 池为空时自动创建新实例，超过容量时丢弃归还的实例。
struct WasmInstancePool {
    /// 实例池
    pool: ArrayQueue<(Store<WasmStoreCtx>, Instance, CachedFunc)>,
    /// 已编译模块
    module: Arc<Module>,
    /// 预注册宿主函数的 Linker 模板，clone 时仅增加 Arc 引用计数，
    /// 避免每次池 miss 都重新注册全部宿主函数。
    linker_template: Linker<WasmStoreCtx>,
}

impl WasmInstancePool {
    fn new(module: Arc<Module>, max_size: usize) -> Result<Self, PluginError> {
        let linker_template = create_linker()?;
        Ok(Self {
            pool: ArrayQueue::new(max_size),
            module,
            linker_template,
        })
    }

    /// 从池中获取一个 (Store, Instance, CachedFunc)，池为空时创建新的
    async fn acquire(&self) -> Result<(Store<WasmStoreCtx>, Instance, CachedFunc), PluginError> {
        // 尝试从池中获取
        if let Some((store, instance, func)) = self.pool.pop() {
            return Ok((store, instance, func));
        }

        // 池 miss：clone Linker（仅 Arc 引用计数 +1），避免重新注册宿主函数
        let mut store = create_store();
        let mut linker = self.linker_template.clone();
        linker
            .define_unknown_imports_as_default_values(&mut store, &self.module)
            .map_err(|e| PluginError::LoadError(format!("stub unknown imports failed: {}", e)))?;
        let instance = linker
            .instantiate_async(&mut store, &self.module)
            .await
            .map_err(|e| PluginError::ExecuteError(format!("instantiate failed: {}", e)))?;

        let func = CachedFunc::resolve(&mut store, &instance)?;
        Ok((store, instance, func))
    }

    fn release(&self, store: Store<WasmStoreCtx>, instance: Instance, func: CachedFunc) {
        // 队列满则丢弃（ArrayQueue 满时返回 Err，自动清理）
        let _ = self.pool.push((store, instance, func));
    }
}

/// WASM 插件包装器
pub struct WasmPlugin {
    /// 插件名称
    plugin_name: String,
    /// 插件信息
    plugin_info: PluginInfo,
    /// 实例池，每个插件一个
    pool: Arc<WasmInstancePool>,
    /// 缓存序列化后的 config 字符串，避免每次 hook 调用重复的序列化。
    /// 使用 Arc<str> 避免 clone 时的堆分配。
    cached_config: OnceLock<Arc<str>>,
}

impl WasmPlugin {
    /// 从 WASM 字节码加载插件
    pub fn from_bytes(wasm_bytes: &[u8]) -> Result<Self, PluginError> {
        let engine = &*ENGINE;
        let module = Module::new(engine, wasm_bytes)
            .map_err(|e| PluginError::LoadError(format!("compile wasm failed: {}", e)))?;

        let (name, info) = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(Self::load_info(&module))
        })?;

        // 预解析 imports，池扩容到 128 减少高并发下的 miss 率
        let pool = Arc::new(WasmInstancePool::new(Arc::new(module), 128)?);

        Ok(Self {
            plugin_name: name,
            plugin_info: info,
            pool,
            cached_config: OnceLock::new(),
        })
    }

    /// 从 WASM 文件路径加载插件
    pub fn from_file(path: &std::path::Path) -> Result<Self, PluginError> {
        let bytes = fs::read(path)
            .map_err(|e| PluginError::LoadError(format!("read wasm file failed: {}", e)))?;
        Self::from_bytes(&bytes)
    }

    /// 调用 WASM 插件元信息导出
    async fn load_info(module: &Module) -> Result<(String, PluginInfo), PluginError> {
        let engine = &*ENGINE;
        let mut store = Store::new(engine, ());
        let linker = create_load_linker(&mut store, module)?;
        let instance = linker
            .instantiate_async(&mut store, module)
            .await
            .map_err(|e| PluginError::LoadError(format!("instantiate failed: {}", e)))?;

        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| PluginError::LoadError("wasm module has no 'memory' export".into()))?;

        let info_fn = instance
            .get_typed_func::<(), i64>(&mut store, "plugin_info")
            .map_err(|e| PluginError::LoadError(format!("get plugin_info export failed: {}", e)))?;

        let result = info_fn
            .call_async(&mut store, ())
            .await
            .map_err(|e| PluginError::LoadError(format!("call plugin_info failed: {}", e)))?;

        let ptr = (result >> 32) as usize;
        let len = (result & 0xFFFFFFFF) as usize;

        let mut buf = vec![0u8; len];
        memory.read(&store, ptr, &mut buf).map_err(|e| {
            PluginError::LoadError(format!("read plugin_info result failed: {}", e))
        })?;

        // 释放 WASM 侧 plugin_info 分配的内存
        if let Ok(dealloc_fn) =
            instance.get_typed_func::<(i32, i32), ()>(&mut store, "aiway_dealloc")
        {
            let _ = dealloc_fn
                .call_async(&mut store, (ptr as i32, len as i32))
                .await;
        }

        let wasm_info: WasmPluginInfo = bincode::deserialize(&buf).map_err(|e| {
            PluginError::LoadError(format!("deserialize plugin_info failed: {}", e))
        })?;

        let version = wasm_info
            .version
            .parse::<Version>()
            .map_err(|e| PluginError::LoadError(format!("parse version failed: {}", e)))?;

        Ok((
            wasm_info.name,
            PluginInfo {
                version,
                default_config: serde_json::from_str(&wasm_info.default_config).map_err(|e| {
                    PluginError::LoadError(format!("parse default_config failed: {}", e))
                })?,
                description: wasm_info.description,
                readme: wasm_info.readme,
            },
        ))
    }

    /// 调用 WASM 插件的指定 Hook
    ///
    /// 调用前注入 HttpContext 指针，调用后清除，确保宿主函数可访问真实请求数据。
    async fn call_wasm(
        &self,
        hook_id: i32,
        input: &WasmInput,
        ctx: &mut HttpContext,
    ) -> Result<WasmOutput, PluginError> {
        let (mut store, instance, func) = self.pool.acquire().await?;

        // 注入 HttpContext 指针和插件名
        unsafe {
            *store.data().http_ctx.get() = Some(NonNull::from(ctx));
            *store.data().plugin_name.get() = Some(NonNull::from(self.plugin_name.as_str()));
        }
        let result = self.execute_wasm(hook_id, input, &mut store, &func).await;
        // 清除指针
        unsafe {
            *store.data().http_ctx.get() = None;
            *store.data().plugin_name.get() = None;
        }

        // 无论成功失败，都归还到池
        self.pool.release(store, instance, func);

        result
    }

    /// 实际执行 WASM 调用
    async fn execute_wasm(
        &self,
        hook_id: i32,
        input: &WasmInput,
        store: &mut Store<WasmStoreCtx>,
        func: &CachedFunc,
    ) -> Result<WasmOutput, PluginError> {
        // 序列化输入
        let input_bytes = bincode::serialize(input)
            .map_err(|e| PluginError::SerializeError(format!("serialize input failed: {}", e)))?;

        // 在 WASM 内存中分配空间并写入input数据
        let input_ptr = func
            .alloc_fn
            .call_async(&mut *store, input_bytes.len() as i32)
            .await
            .map_err(|e| PluginError::ExecuteError(format!("aiway_alloc failed: {}", e)))?;

        func.memory
            .write(&mut *store, input_ptr as usize, &input_bytes)
            .map_err(|e| {
                PluginError::ExecuteError(format!("write input to memory failed: {}", e))
            })?;

        // 调用插件钩子函数（使用缓存的 call_fn）
        let call_result = func
            .call_fn
            .call_async(&mut *store, (hook_id, input_ptr, input_bytes.len() as i32))
            .await;

        // 无论 call_fn 成功或 trap，都释放 input_ptr
        if let Some(dealloc_fn) = &func.dealloc_fn {
            let _ = dealloc_fn
                .call_async(&mut *store, (input_ptr, input_bytes.len() as i32))
                .await;
        }

        let result = call_result
            .map_err(|e| PluginError::ExecuteError(format!("aiway_call trap: {}", e)))?;

        // 解码返回值
        let result_ptr = (result >> 32) as usize;
        let result_len = (result & 0xFFFFFFFF) as usize;

        // 用 result 变量保存最终结果，确保 result_ptr 的 dealloc 在 return 之前执行
        let output_result = if result_ptr == 0 {
            let mut err_buf = vec![0u8; result_len];
            func.memory
                .read(&*store, 1, &mut err_buf)
                .map_err(|e| PluginError::ExecuteError(format!("read error msg failed: {}", e)))?;

            // 尝试解析 Reject 格式：前 4 字节为 u32 BE 状态码，>0 表示 Reject
            if err_buf.len() >= 4 {
                let status = u32::from_be_bytes([err_buf[0], err_buf[1], err_buf[2], err_buf[3]]);
                if status > 0 && status <= 599 {
                    let message = String::from_utf8_lossy(&err_buf[4..]).to_string();
                    Err(PluginError::Reject(status as u16, message))
                } else {
                    Err(PluginError::ExecuteError(
                        String::from_utf8_lossy(&err_buf).to_string(),
                    ))
                }
            } else {
                Err(PluginError::ExecuteError(
                    String::from_utf8_lossy(&err_buf).to_string(),
                ))
            }
        } else {
            let mut result_buf = vec![0u8; result_len];
            func.memory
                .read(&*store, result_ptr, &mut result_buf)
                .map_err(|e| PluginError::ExecuteError(format!("read result failed: {}", e)))?;

            bincode::deserialize(&result_buf).map_err(|e| {
                PluginError::SerializeError(format!("deserialize output failed: {}", e))
            })
        };

        // 释放 result_ptr 的 WASM 内存（在 return 之前统一释放）
        if result_ptr > 0
            && let Some(dealloc_fn) = &func.dealloc_fn
        {
            let _ = dealloc_fn
                .call_async(&mut *store, (result_ptr as i32, result_len as i32))
                .await;
        }

        output_result
    }
}

#[async_trait]
impl Plugin for WasmPlugin {
    fn name(&self) -> &str {
        &self.plugin_name
    }

    fn info(&self) -> PluginInfo {
        self.plugin_info.clone()
    }

    async fn on_request(
        &self,
        config: &Value,
        head: &mut request::Parts,
        ctx: &mut dyn PluginContext,
    ) -> Result<(), PluginError> {
        let http_ctx = ctx.as_any_mut().downcast_mut::<HttpContext>().unwrap();
        let input = WasmInput {
            config: self
                .cached_config
                .get_or_init(|| serde_json::to_string(config).unwrap_or_default().into())
                .clone(),
            head: Some(WasmHead::from_request_parts(head)),
            body: None,
            request_id: None,
            request_ts: None,
        };

        let output = self.call_wasm(HOOK_ON_REQUEST, &input, http_ctx).await?;

        if let Some(modified_head) = output.head {
            modified_head.apply_to_request_parts(head);
        }

        Ok(())
    }

    async fn on_request_body(
        &self,
        config: &Value,
        body: &mut Option<Bytes>,
        ctx: &mut dyn PluginContext,
    ) -> Result<(), PluginError> {
        let http_ctx = ctx.as_any_mut().downcast_mut::<HttpContext>().unwrap();
        let input = WasmInput {
            config: self
                .cached_config
                .get_or_init(|| serde_json::to_string(config).unwrap_or_default().into())
                .clone(),
            head: None,
            body: body.as_ref().map(|b| b.to_vec()),
            request_id: None,
            request_ts: None,
        };

        let output = self
            .call_wasm(HOOK_ON_REQUEST_BODY, &input, http_ctx)
            .await?;

        if let Some(modified_body) = output.body {
            *body = Some(Bytes::from(modified_body));
        }

        Ok(())
    }

    async fn on_response(
        &self,
        config: &Value,
        head: &mut response::Parts,
        ctx: &mut dyn PluginContext,
    ) -> Result<(), PluginError> {
        let http_ctx = ctx.as_any_mut().downcast_mut::<HttpContext>().unwrap();
        let input = WasmInput {
            config: self
                .cached_config
                .get_or_init(|| serde_json::to_string(config).unwrap_or_default().into())
                .clone(),
            head: Some(WasmHead::from_response_parts(head)),
            body: None,
            request_id: None,
            request_ts: None,
        };

        let output = self.call_wasm(HOOK_ON_RESPONSE, &input, http_ctx).await?;

        if let Some(modified_head) = output.head {
            modified_head.apply_to_response_parts(head);
        }

        Ok(())
    }

    async fn on_response_body(
        &self,
        config: &Value,
        body: &mut Option<Bytes>,
        ctx: &mut dyn PluginContext,
    ) -> Result<(), PluginError> {
        let http_ctx = ctx.as_any_mut().downcast_mut::<HttpContext>().unwrap();
        let input = WasmInput {
            config: self
                .cached_config
                .get_or_init(|| serde_json::to_string(config).unwrap_or_default().into())
                .clone(),
            head: None,
            body: body.as_ref().map(|b| b.to_vec()),
            request_id: None,
            request_ts: None,
        };

        let output = self
            .call_wasm(HOOK_ON_RESPONSE_BODY, &input, http_ctx)
            .await?;

        if let Some(modified_body) = output.body {
            *body = Some(Bytes::from(modified_body));
        }

        Ok(())
    }

    async fn on_logging(&self, config: &Value, ctx: &mut dyn PluginContext) {
        let http_ctx = ctx.as_any_mut().downcast_mut::<HttpContext>().unwrap();
        let input = WasmInput {
            config: self
                .cached_config
                .get_or_init(|| serde_json::to_string(config).unwrap_or_default().into())
                .clone(),
            head: None,
            body: None,
            request_id: Some(http_ctx.request_id()),
            request_ts: Some(http_ctx.request_ts()),
        };

        let _ = self.call_wasm(HOOK_ON_LOGGING, &input, http_ctx).await;
    }
}

// ----------------------------------------------------------------------
// 从 URL 加载插件
// ----------------------------------------------------------------------

/// 从指定 URL 下载并加载 WASM 插件
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

        let plugin = WasmPlugin::from_bytes(&bytes)?;
        Ok(Box::new(plugin))
    }
}

/// 从 WASM 字节码创建插件实例
pub fn plugin_from_bytes(bytes: &[u8]) -> Result<Box<dyn Plugin>, PluginError> {
    let plugin = WasmPlugin::from_bytes(bytes)?;
    Ok(Box::new(plugin))
}
