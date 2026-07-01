use tracing_subscriber::EnvFilter;

/// Initializes the global tracing subscriber.
pub fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("warn,blog_server=info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .init();
}
