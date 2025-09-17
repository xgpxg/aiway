pub fn init_log() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "info,rocket=warn,rocket::response::debug=error,rocket::launch=error".into()
            }),
        )
        .with_level(true)
        .with_ansi(true)
        .with_line_number(true)
        .with_timer(tracing_subscriber::fmt::time::ChronoLocal::new(
            "%Y-%m-%d %H:%M:%S.%3f".to_string(),
        ))
        .compact() // 避免乱码问题
        .init();
}
