use std::collections::HashSet;
use std::sync::LazyLock;

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
