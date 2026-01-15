use std::collections::HashSet;
use std::sync::LazyLock;

/// APIKey加密默认密钥
pub const ENCRYPT_KEY: &[u8; 32] = b"00000000000000000000000000000000";

/// 状态上报间隔秒数
pub const REPORT_STATE_INTERVAL: u64 = 5;

/// 禁止透传的HTTP头部
pub static BAN_HEADERS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    [
        "content-length",
        "x-frame-options",
        "x-content-type-options",
        "x-xss-protection",
        "server",
        "transfer-encoding",
        "connection",
        "te",
        "trailer",
        "permissions-policy",
    ]
    .iter()
    .copied()
    .collect()
});
