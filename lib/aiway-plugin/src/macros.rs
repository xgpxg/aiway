//! WASM 插件导出宏
//!
//! 为插件开发者生成 C ABI 导出函数，供网关 wasmtime 运行时调用。

/// 格式化日志宏（ERROR 级别）
///
/// # 用法
/// ```ignore
/// log_error!(ctx, "request failed: {}", err);
/// ```
#[macro_export]
macro_rules! log_error {
    ($ctx:expr, $($arg:tt)*) => {
        $ctx.log_error(&format!($($arg)*))
    };
}

/// WARN日志
#[macro_export]
macro_rules! log_warn {
    ($ctx:expr, $($arg:tt)*) => {
        $ctx.log_warn(&format!($($arg)*))
    };
}

/// INFO日志
#[macro_export]
macro_rules! log_info {
    ($ctx:expr, $($arg:tt)*) => {
        $ctx.log_info(&format!($($arg)*))
    };
}

/// DEBUG日志
#[macro_export]
macro_rules! log_debug {
    ($ctx:expr, $($arg:tt)*) => {
        $ctx.log_debug(&format!($($arg)*))
    };
}

/// TRACE日志
#[macro_export]
macro_rules! log_trace {
    ($ctx:expr, $($arg:tt)*) => {
        $ctx.log_trace(&format!($($arg)*))
    };
}

/// 导出 WASM 插件
/// # 用法
/// ```ignore
/// struct MyPlugin;
/// impl aiway_plugin::Plugin for MyPlugin { /* ... */ }
///
/// aiway_plugin::export_wasm!(MyPlugin);
/// ```
#[macro_export]
macro_rules! export_wasm {
    ($plugin_type:ty) => {
        /// 插件实例（全局单例，因为 WASM 插件应无状态）
        static PLUGIN: std::sync::LazyLock<$plugin_type> =
            std::sync::LazyLock::new(|| <$plugin_type>::new());

        // /// 分配内存（供 Host 写入输入数据）
        // #[unsafe(no_mangle)]
        // pub extern "C" fn aiway_alloc(size: i32) -> i32 {
        //     let layout = std::alloc::Layout::from_size_align(size as usize, 1).unwrap();
        //     unsafe {
        //         let ptr = std::alloc::alloc(layout);
        //         ptr as i32
        //     }
        // }

        /// 释放内存
        #[unsafe(no_mangle)]
        pub extern "C" fn aiway_dealloc(ptr: i32, size: i32) {
            let layout = std::alloc::Layout::from_size_align(size as usize, 1).unwrap();
            unsafe {
                std::alloc::dealloc(ptr as *mut u8, layout);
            }
        }

        /// 返回插件元数据
        ///
        /// 返回 i64，高 32 位 = 数据指针，低 32 位 = 数据长度
        #[unsafe(no_mangle)]
        pub extern "C" fn plugin_info() -> i64 {
            let info = aiway_plugin::wasm_types::WasmPluginInfo {
                name: PLUGIN.name().to_string(),
                version: PLUGIN.info().version.to_string(),
                description: PLUGIN.info().description.clone(),
                default_config: aiway_plugin::serde_json::to_string(&PLUGIN.info().default_config)
                    .unwrap_or_default(),
                readme: PLUGIN.info().readme.clone(),
            };

            let bytes = $crate::bincode::serialize(&info).unwrap();
            let len = bytes.len();
            let ptr = bytes.as_ptr() as i32;
            std::mem::forget(bytes);

            ((ptr as i64) << 32) | (len as i64)
        }

        /// 插件Hook调用入口
        ///
        /// # 参数
        /// - `hook_id`: Hook ID（见 `wasm_types` 常量）
        ///
        /// 插件输入数据（config/body）通过宿主函数从 HttpContext 按需获取，不再经参数传递。
        ///
        /// # 返回
        /// i64，高 32 位 = 状态标记（0 = 成功，非 0 = 错误），
        /// 成功时低 32 位 = 控制流（0 = Continue，1 = Respond），
        /// 错误时低 32 位 = 错误信息长度（数据写入固定地址 [`ERROR_BUF_PTR`](aiway_plugin::wasm_types::ERROR_BUF_PTR)）。
        #[unsafe(no_mangle)]
        pub extern "C" fn aiway_call(hook_id: i32) -> i64 {
            // 根据 hook_id 分发
            let result: Result<aiway_plugin::Outcome, String> = match hook_id {
                aiway_plugin::wasm_types::HOOK_ON_REQUEST => handle_on_request(&PLUGIN),
                aiway_plugin::wasm_types::HOOK_ON_REQUEST_BODY => handle_on_request_body(&PLUGIN),
                aiway_plugin::wasm_types::HOOK_ON_RESPONSE => handle_on_response(&PLUGIN),
                aiway_plugin::wasm_types::HOOK_ON_RESPONSE_BODY => handle_on_response_body(&PLUGIN),
                aiway_plugin::wasm_types::HOOK_ON_LOGGING => handle_on_logging(&PLUGIN),
                _ => Err(format!("unknown hook_id: {}", hook_id)),
            };

            match result {
                Ok(outcome) => encode_outcome(&outcome),
                Err(err_msg) => encode_error(&err_msg),
            }
        }

        /// 编码成功控制流：高位 0 = 成功（无错误），低位为 [`HookControl`](aiway_plugin::wasm_types::HookControl)
        fn encode_outcome(outcome: &aiway_plugin::Outcome) -> i64 {
            match outcome {
                aiway_plugin::Outcome::Continue => {
                    aiway_plugin::wasm_types::HookControl::Continue as i64
                }
                aiway_plugin::Outcome::Respond(resp) => {
                    // 主动响应数据写入 Host 侧 HttpContext，仅返回 Respond 控制流标记
                    aiway_plugin::respond_to_host(
                        resp.status,
                        resp.headers.clone(),
                        resp.body.clone(),
                    );
                    aiway_plugin::wasm_types::HookControl::Respond as i64
                }
            }
        }

        /// 编码错误信息：写入固定地址，返回高位非 0 标记错误（低位为错误信息长度）
        fn encode_error(msg: &str) -> i64 {
            let bytes = msg.as_bytes();
            // 将错误信息写入 ERROR_BUF_PTR 处
            let dst = unsafe {
                std::slice::from_raw_parts_mut(
                    aiway_plugin::wasm_types::ERROR_BUF_PTR as *mut u8,
                    bytes.len(),
                )
            };
            dst.copy_from_slice(bytes);
            (1i64 << 32) | (bytes.len() as i64)
        }

        /// 将 PluginError 编码为错误消息
        fn encode_plugin_error(e: aiway_plugin::PluginError) -> String {
            format!("{}", e)
        }

        /// 处理 on_request
        fn handle_on_request(plugin: &$plugin_type) -> Result<aiway_plugin::Outcome, String> {
            let mut ctx = aiway_plugin::WasmHttpContext;

            aiway_plugin::block_on(async { plugin.on_request(&mut ctx).await })
                .map_err(encode_plugin_error)
        }

        /// 处理 on_request_body
        fn handle_on_request_body(plugin: &$plugin_type) -> Result<aiway_plugin::Outcome, String> {
            let mut ctx = aiway_plugin::WasmHttpContext;

            aiway_plugin::block_on(async { plugin.on_request_body(&mut ctx).await })
                .map_err(encode_plugin_error)
        }

        /// 处理 on_response
        fn handle_on_response(plugin: &$plugin_type) -> Result<aiway_plugin::Outcome, String> {
            let mut ctx = aiway_plugin::WasmHttpContext;

            aiway_plugin::block_on(async { plugin.on_response(&mut ctx).await })
                .map_err(encode_plugin_error)
        }

        /// 处理 on_response_body
        fn handle_on_response_body(plugin: &$plugin_type) -> Result<aiway_plugin::Outcome, String> {
            let mut ctx = aiway_plugin::WasmHttpContext;

            aiway_plugin::block_on(async { plugin.on_response_body(&mut ctx).await })
                .map_err(encode_plugin_error)
        }

        /// 处理 on_logging
        fn handle_on_logging(plugin: &$plugin_type) -> Result<aiway_plugin::Outcome, String> {
            let mut ctx = aiway_plugin::WasmHttpContext;

            aiway_plugin::block_on(async {
                plugin.on_logging(&mut ctx).await;
            });

            Ok(aiway_plugin::Outcome::Continue)
        }
    };
}
