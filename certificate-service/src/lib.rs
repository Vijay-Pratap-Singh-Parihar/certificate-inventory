#![deny(elided_lifetimes_in_paths)]

pub mod adapters;
pub mod application;
pub mod config;
pub mod domain;
pub mod infrastructure;

#[cfg(test)]
mod tests;

pub use application::usecases::UseCases;
pub use config::AppConfig;
pub use infrastructure::shared::{build_use_cases, Repositories};

/// Initialize tracing logger (env filter from RUST_LOG).
pub fn init_logger() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,certificate_service=debug")),
        )
        .try_init()?;
    Ok(())
}

/// Macro to wire AppConfig, Repositories, and UseCases (mirrors telephony setup_dependencies!).
/// Returns a future; use `.await` at the call site. Runs on the existing Tokio runtime.
#[macro_export]
macro_rules! setup_dependencies {
    () => {{
        use std::sync::Arc;

        async move {
            let app_config = Arc::new(AppConfig::new().expect("Failed to load application config"));
            let repositories = Arc::new(Repositories::new(Arc::clone(&app_config)).await);
            let use_cases = Arc::new(build_use_cases(repositories.as_ref()));
            (app_config, repositories, use_cases)
        }
    }};

    ($init:expr) => {{
        async move {
            let (app_config, repositories, use_cases) = setup_dependencies!().await;
            $init(app_config, repositories, use_cases)
        }
    }};
}
