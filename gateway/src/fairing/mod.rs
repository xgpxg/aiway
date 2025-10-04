//! # 全局fairing
//!
//! faring分为两个阶段：
//! 1. 收到请求，到达API处理端点前
//! 2. API端点处理完成，响应客户端前
//!
//! 这两个阶段，通过Filter的实现扩展，进行拦截处理。
//!
//! 在第一阶段，即前置处理阶段，按顺序执行已配置的插件，并传递给下一个插件。
//! 返回Ok：继续执行下一个插件
//! 返回Err：终止执行，修改请求的uri，强制转发到一个特殊的端点。（或者考虑由配置决定是否终止流程）
//!
//! 在第二阶段，即后置处理阶段，按顺序执行已配置的插件，并传递给下一个插件。
//! 可在此阶段修改响应结果。
//!
pub mod auth;
pub mod catchers;
pub mod cleanup;
pub mod filter;
pub mod global_filter;
pub mod lb;
pub mod logger;
pub mod request;
pub mod response;
pub mod routing;
pub mod security;

/// 在请求fairing阶段，提取不含网关前缀的API路径
///
/// 如果原始路径不含前缀，则直接返回，不执行后续逻辑。
///
/// **废弃**
#[deprecated]
#[macro_export]
macro_rules! extract_api_path {
    ($req:expr) => {{
        use $crate::constants;
        let uri_path = $req.uri().path().as_str();
        match uri_path.strip_prefix(constants::API_PREFIX) {
            Some(path) => path,
            None => return,
        }
    }};
}

/// 在request fairing执行期间设置错误信息。通过自定义的Header在后续流程中传递。
///
/// 接收以下3个参数：
/// - req: rocket的请求对象
/// - code: 错误码，必须为已知的标准http状态码
/// - msg: 错误信息
/// 当设置了错误信息后，后续的fairing将被跳过，然后会被catchers拦截，
/// 在catchers中提取状态码和错误信息后返回给客户端。
#[macro_export]
macro_rules! set_error {
    ($req:expr, $code:expr, $msg:expr) => {{
        $req.add_header(rocket::http::Header::new(
            $crate::context::Headers::ERROR_CODE,
            $code.to_string(),
        ));
        $req.add_header(rocket::http::Header::new(
            $crate::context::Headers::ERROR_MESSAGE,
            $msg,
        ));
    }};
}

/// 在request fairing执行期间，检查是否已设置错误信息。
/// 如果已设置错误信息，则直接返回，不执行后续逻辑。
#[macro_export]
macro_rules! skip_if_error {
    ($req:expr) => {{
        if $req.headers().get_one("X-Error-Code").is_some() {
            return;
        }
    }};
}

/// 提取错误信息。
/// 返回一个元组，包含错误码和错误信息。
/// 如果未设置错误信息，即正常状态，则返回None。
#[macro_export]
macro_rules! extract_error {
    ($req:expr) => {{
        if let Some(code) = $req.headers().get_one("X-Error-Code") {
            Some((
                code.parse::<u16>().unwrap(),
                $req.headers()
                    .get_one("X-Error-Message")
                    .unwrap_or_default(),
            ))
        } else {
            None
        }
    }};
}
