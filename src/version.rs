use std::sync::LazyLock;

/// Defines the application version.
pub static VERSION: LazyLock<String> = LazyLock::new(|| {
    format!(
        "v{}-{}",
        env!("CARGO_PKG_VERSION"),
        option_env!("GIT_COMMIT").unwrap_or("unknown")
    )
});
