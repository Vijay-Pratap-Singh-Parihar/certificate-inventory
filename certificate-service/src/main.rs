use certificate_service::{build_use_cases, init_logger, setup_dependencies, AppConfig, Repositories};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    init_logger()?;

    let (app_config, _repositories, use_cases) = setup_dependencies!().await;
    info!(
        "Certificate service starting (config loaded, port from env or default)"
    );

    let port = app_config.port();
    let app = certificate_service::infrastructure::apis::create_router(use_cases);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    info!(%addr, "listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
