//! WASM 插件导出宏
//!
//! 为插件开发者生成 C ABI 导出函数，供网关 wasmtime 运行时调用。

/// 导出 WASM 插件
///
/// 生成以下导出函数：
/// - `plugin_info()` -> i64：返回插件元信息（bincode 编码）
/// - `aiway_call(hook_id, input_ptr, input_len)` -> i64：插件钩子调用入口
/// - `aiway_alloc(size)` -> i32：内存分配
/// - `aiway_dealloc(ptr, size)`：内存释放
///
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

        /// 分配内存（供 Host 写入输入数据）
        #[unsafe(no_mangle)]
        pub extern "C" fn aiway_alloc(size: i32) -> i32 {
            let layout = std::alloc::Layout::from_size_align(size as usize, 1).unwrap();
            unsafe {
                let ptr = std::alloc::alloc(layout);
                ptr as i32
            }
        }

        /// 释放内存
        #[unsafe(no_mangle)]
        pub extern "C" fn aiway_dealloc(ptr: i32, size: i32) {
            let layout = std::alloc::Layout::from_size_align(size as usize, 1).unwrap();
            unsafe {
                std::alloc::dealloc(ptr as *mut u8, layout);
            }
        }

        /// 返回插件元信息
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
            };

            let bytes = bincode::serialize(&info).unwrap();
            let len = bytes.len();
            let ptr = bytes.as_ptr() as i32;
            std::mem::forget(bytes);

            ((ptr as i64) << 32) | (len as i64)
        }

        /// 插件钩子调用入口
        ///
        /// # 参数
        /// - `hook_id`: 钩子 ID（见 `wasm_types` 常量）
        /// - `input_ptr`: 输入数据在 WASM 内存中的指针
        /// - `input_len`: 输入数据长度
        ///
        /// # 返回
        /// i64，高 32 位 = 结果指针，低 32 位 = 结果长度。
        /// 若高 32 位为 0，表示错误，低 32 位是错误信息长度（数据在 ptr=1 处）。
        #[unsafe(no_mangle)]
        pub extern "C" fn aiway_call(hook_id: i32, input_ptr: i32, input_len: i32) -> i64 {
            // 读取输入数据
            let input_slice =
                unsafe { std::slice::from_raw_parts(input_ptr as *const u8, input_len as usize) };

            let input: aiway_plugin::wasm_types::WasmInput = match bincode::deserialize(input_slice)
            {
                Ok(v) => v,
                Err(e) => return encode_error(&format!("deserialize input failed: {}", e)),
            };

            // 根据 hook_id 分发
            let result = match hook_id {
                aiway_plugin::wasm_types::HOOK_ON_REQUEST => {
                    handle_on_request(&PLUGIN, &input)
                }
                aiway_plugin::wasm_types::HOOK_ON_REQUEST_BODY => {
                    handle_on_request_body(&PLUGIN, &input)
                }
                aiway_plugin::wasm_types::HOOK_ON_RESPONSE => {
                    handle_on_response(&PLUGIN, &input)
                }
                aiway_plugin::wasm_types::HOOK_ON_RESPONSE_BODY => {
                    handle_on_response_body(&PLUGIN, &input)
                }
                aiway_plugin::wasm_types::HOOK_ON_LOGGING => {
                    handle_on_logging(&PLUGIN, &input)
                }
                _ => Err(format!("unknown hook_id: {}", hook_id)),
            };

            match result {
                Ok(output) => encode_output(&output),
                Err(err_msg) => encode_error(&err_msg),
            }
        }

        /// 编码成功输出
        fn encode_output(
            output: &aiway_plugin::wasm_types::WasmOutput,
        ) -> i64 {
            match bincode::serialize(output) {
                Ok(bytes) => {
                    let len = bytes.len();
                    let ptr = bytes.as_ptr() as i32;
                    std::mem::forget(bytes);
                    ((ptr as i64) << 32) | (len as i64)
                }
                Err(e) => encode_error(&format!("serialize output failed: {}", e)),
            }
        }

        /// 编码错误信息（ptr=0, len=错误信息长度）
        fn encode_error(msg: &str) -> i64 {
            let bytes = msg.as_bytes();
            // 将错误信息写入 ptr=1 处（预留 1 字节作为 null 标记）
            let dst = unsafe { std::slice::from_raw_parts_mut(1 as *mut u8, bytes.len()) };
            dst.copy_from_slice(bytes);
            (0i64 << 32) | (bytes.len() as i64)
        }

        /// 解析 JSON 配置字符串
        fn parse_config(config_str: &str) -> Result<aiway_plugin::serde_json::Value, String> {
            aiway_plugin::serde_json::from_str(config_str)
                .map_err(|e| format!("parse config failed: {}", e))
        }

        /// 处理 on_request
        fn handle_on_request(
            plugin: &$plugin_type,
            input: &aiway_plugin::wasm_types::WasmInput,
        ) -> Result<aiway_plugin::wasm_types::WasmOutput, String> {
            let config = parse_config(&input.config)?;
            let mut dummy_head = build_dummy_request_parts(input.head.as_ref().unwrap());
            let mut ctx = aiway_plugin::WasmHttpContext;

            aiway_plugin::block_on(async {
                plugin
                    .on_request(&config, &mut dummy_head, &mut ctx)
                    .await
            })
            .map_err(|e| format!("{}", e))?;

            let output_head =
                aiway_plugin::wasm_types::WasmHead::from_request_parts(&dummy_head);

            Ok(aiway_plugin::wasm_types::WasmOutput {
                head: Some(output_head),
                body: None,
            })
        }

        /// 处理 on_request_body
        fn handle_on_request_body(
            plugin: &$plugin_type,
            input: &aiway_plugin::wasm_types::WasmInput,
        ) -> Result<aiway_plugin::wasm_types::WasmOutput, String> {
            let config = parse_config(&input.config)?;
            let mut body = input.body.as_ref().map(|b| aiway_plugin::Bytes::from(b.clone()));
            let mut ctx = aiway_plugin::WasmHttpContext;

            aiway_plugin::block_on(async {
                plugin
                    .on_request_body(&config, &mut body, &mut ctx)
                    .await
            })
            .map_err(|e| format!("{}", e))?;

            Ok(aiway_plugin::wasm_types::WasmOutput {
                head: None,
                body: body.map(|b| b.to_vec()),
            })
        }

        /// 处理 on_response
        fn handle_on_response(
            plugin: &$plugin_type,
            input: &aiway_plugin::wasm_types::WasmInput,
        ) -> Result<aiway_plugin::wasm_types::WasmOutput, String> {
            let config = parse_config(&input.config)?;
            let mut dummy_head = build_dummy_response_parts(input.head.as_ref().unwrap());
            let mut ctx = aiway_plugin::WasmHttpContext;

            aiway_plugin::block_on(async {
                plugin
                    .on_response(&config, &mut dummy_head, &mut ctx)
                    .await
            })
            .map_err(|e| format!("{}", e))?;

            let output_head =
                aiway_plugin::wasm_types::WasmHead::from_response_parts(&dummy_head);

            Ok(aiway_plugin::wasm_types::WasmOutput {
                head: Some(output_head),
                body: None,
            })
        }

        /// 处理 on_response_body
        /// 因为 pingora 仅支持同步的，所以这里也需要同步
        fn handle_on_response_body(
            plugin: &$plugin_type,
            input: &aiway_plugin::wasm_types::WasmInput,
        ) -> Result<aiway_plugin::wasm_types::WasmOutput, String> {
            let config = parse_config(&input.config)?;
            let mut body = input.body.as_ref().map(|b| aiway_plugin::Bytes::from(b.clone()));
            let mut ctx = aiway_plugin::WasmHttpContext;

            plugin
                .on_response_body(&config, &mut body, &mut ctx)
                .map_err(|e| format!("{}", e))?;

            Ok(aiway_plugin::wasm_types::WasmOutput {
                head: None,
                body: body.map(|b| b.to_vec()),
            })
        }

        /// 处理 on_logging
        fn handle_on_logging(
            plugin: &$plugin_type,
            input: &aiway_plugin::wasm_types::WasmInput,
        ) -> Result<aiway_plugin::wasm_types::WasmOutput, String> {
            let config = parse_config(&input.config)?;
            let mut ctx = aiway_plugin::WasmHttpContext;

            aiway_plugin::block_on(async {
                plugin.on_logging(&config, &mut ctx).await;
            });

            Ok(aiway_plugin::wasm_types::WasmOutput {
                head: None,
                body: None,
            })
        }

        /// 构建虚拟的请求 Parts（用于传递给插件）
        fn build_dummy_request_parts(
            head: &aiway_plugin::wasm_types::WasmHead,
        ) -> aiway_plugin::http::request::Parts {
            let method: aiway_plugin::http::Method = head
                .method
                .as_deref()
                .unwrap_or("GET")
                .parse()
                .unwrap_or(aiway_plugin::http::Method::GET);

            let uri: aiway_plugin::http::Uri = head
                .uri
                .as_deref()
                .unwrap_or("/")
                .parse()
                .unwrap_or_default();

            let mut headers = aiway_plugin::http::HeaderMap::new();
            for (k, v) in &head.headers {
                if let (Ok(name), Ok(value)) = (
                    aiway_plugin::http::HeaderName::from_bytes(k.as_bytes()),
                    aiway_plugin::http::HeaderValue::from_str(v),
                ) {
                    headers.insert(name, value);
                }
            }

            let (mut parts, _) = aiway_plugin::http::Request::builder()
                .method(method)
                .uri(uri)
                .body(())
                .unwrap()
                .into_parts();
            parts.headers = headers;
            parts
        }

        /// 构建虚拟的响应 Parts（用于传递给插件）
        fn build_dummy_response_parts(
            head: &aiway_plugin::wasm_types::WasmHead,
        ) -> aiway_plugin::http::response::Parts {
            let status = aiway_plugin::http::StatusCode::from_u16(head.status.unwrap_or(200))
                .unwrap_or(aiway_plugin::http::StatusCode::OK);

            let mut headers = aiway_plugin::http::HeaderMap::new();
            for (k, v) in &head.headers {
                if let (Ok(name), Ok(value)) = (
                    aiway_plugin::http::HeaderName::from_bytes(k.as_bytes()),
                    aiway_plugin::http::HeaderValue::from_str(v),
                ) {
                    headers.insert(name, value);
                }
            }

            let (mut parts, _) = aiway_plugin::http::Response::builder()
                .status(status)
                .body(())
                .unwrap()
                .into_parts();
            parts.headers = headers;
            parts
        }
    };
}
