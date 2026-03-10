//! Composition root: Repositories (mirrors telephony infrastructure/shared).
//! UseCases are built here and live in application layer.

use crate::adapters::service_providers::persistence::postgres_certificate_repository::PostgresCertificateRepository;
use crate::application::repositories::certificate_repository_abstract::CertificateRepositoryAbstract;
use crate::application::usecases::UseCases;
use crate::application::utils::certificate_parser_impl::X509CertificateParser;
use crate::config::AppConfig;
use crate::domain::certificate_parser::CertificateParser;
use std::sync::Arc;
use sqlx::postgres::PgPoolOptions;

pub struct Repositories {
    pub certificate_repository: Arc<Box<dyn CertificateRepositoryAbstract>>,
    pub app_config: Arc<AppConfig>,
}

impl Repositories {
    pub async fn new(app_config: Arc<AppConfig>) -> Self {
        const MAX_ATTEMPTS: u32 = 5;
        const RETRY_DELAY: std::time::Duration = std::time::Duration::from_secs(2);

        let mut last_err = None;
        let pool = 'connect: loop {
            for attempt in 1..=MAX_ATTEMPTS {
                let opts = PgPoolOptions::new().max_connections(5);
                println!("DATABASE_URL: {}", app_config.database_url());
                match opts.connect(app_config.database_url()).await {
                    Ok(p) => break 'connect p,
                    Err(e) => {
                        last_err = Some(e);
                        if attempt < MAX_ATTEMPTS {
                            tracing::warn!(
                                attempt,
                                "Postgres connection failed, retrying in {:?}...",
                                RETRY_DELAY
                            );
                            tokio::time::sleep(RETRY_DELAY).await;
                        }
                    }
                }
            }
            panic!(
                "Failed to connect to Postgres: {}",
                last_err
                    .map(|e: sqlx::Error| e.to_string())
                    .unwrap_or_else(|| "unknown".into())
            );
        };

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        let pool = Arc::new(pool);
        let certificate_repository: Arc<Box<dyn CertificateRepositoryAbstract>> =
            Arc::new(Box::new(PostgresCertificateRepository::new(Arc::clone(&pool))));

        Self {
            certificate_repository,
            app_config,
        }
    }
}

/// Build UseCases from Repositories (parser is application-layer impl).
pub fn build_use_cases(repositories: &Repositories) -> UseCases {
    let certificate_parser: Arc<Box<dyn CertificateParser>> =
        Arc::new(Box::new(X509CertificateParser::default()));
    UseCases::new(
        repositories.certificate_repository.clone(),
        certificate_parser,
    )
}
