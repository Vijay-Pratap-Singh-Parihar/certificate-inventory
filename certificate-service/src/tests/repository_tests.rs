//! Repository tests — require DATABASE_URL (e.g. CI or local Postgres).
//! Run with: cargo test repository_tests -- --ignored  (to run only with DB)
//! Or: DATABASE_URL=... cargo test repository_tests

use certificate_service::adapters::service_providers::persistence::postgres_certificate_repository::PostgresCertificateRepository;
use certificate_service::application::repositories::certificate_repository_abstract::CertificateRepositoryAbstract;
use certificate_service::domain::certificate::Certificate;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

async fn test_pool() -> Option<sqlx::PgPool> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let pool = sqlx::PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .ok()?;
    sqlx::migrate!("./migrations").run(&pool).await.ok()?;
    Some(pool)
}

#[tokio::test]
async fn repository_save_and_get_by_id() {
    let pool = match test_pool().await {
        Some(p) => p,
        None => {
            eprintln!("DATABASE_URL not set, skipping repository test");
            return;
        }
    };
    let repo = PostgresCertificateRepository::new(Arc::new(pool));
    let id = Uuid::new_v4().to_string();
    let cert = Certificate::new(
        id.clone(),
        "test.example.com".to_string(),
        "Test CA".to_string(),
        None,
        Utc::now() + chrono::Duration::days(90),
        None,
        vec!["test.example.com".to_string()],
        None,
    );
    let saved = repo.save(cert, "").await;
    assert!(saved.is_ok(), "save failed: {:?}", saved.err());
    let got = repo.get_by_id(&id).await.unwrap();
    assert!(got.is_some());
    let got = got.unwrap();
    assert_eq!(got.id, id);
    assert_eq!(got.subject, "test.example.com");
}

#[tokio::test]
async fn repository_count_total_and_expiring_soon() {
    let pool = match test_pool().await {
        Some(p) => p,
        None => return,
    };
    let repo = PostgresCertificateRepository::new(Arc::new(pool));
    let total = repo.count_total().await;
    assert!(total.is_ok());
    let expiring = repo.count_expiring_soon(30).await;
    assert!(expiring.is_ok());
}

#[tokio::test]
async fn repository_get_metrics() {
    let pool = match test_pool().await {
        Some(p) => p,
        None => return,
    };
    let repo = PostgresCertificateRepository::new(Arc::new(pool));
    let (total, expiring_soon) = repo.get_metrics(30).await.unwrap();
    assert!(total >= 0);
    assert!(expiring_soon >= 0);
}
