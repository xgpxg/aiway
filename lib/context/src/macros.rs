/// 在request fairing执行期间设置错误信息。通过自定义的Header在后续流程中传递。
///
/// 接收以下3个参数：
/// - req: rocket的请求对象
/// - code: 错误码，必须为已知的标准http状态码
/// - msg: 错误信息
///
/// 当设置了错误信息后，后续的fairing将被跳过，然后会被catchers拦截，
/// 在catchers中提取状态码和错误信息后返回给客户端。
#[macro_export]
macro_rules! set_error {
    ($req:expr, $code:expr, $msg:expr) => {{
        $req.add_header(rocket::http::Header::new(
            $crate::Headers::ERROR_CODE,
            $code.to_string(),
        ));
        $req.add_header(rocket::http::Header::new(
            $crate::Headers::ERROR_MESSAGE,
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
