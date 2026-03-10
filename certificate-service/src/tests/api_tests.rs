//! API endpoint tests (POST /certificates, GET /certificates/:id) using mock repository.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use certificate_service::application::repositories::certificate_repository_abstract::{
    CertificateSortBy, CertificateSortOrder, MockCertificateRepositoryAbstract,
};
use certificate_service::application::usecases::UseCases;
use certificate_service::domain::certificate::Certificate;
use certificate_service::domain::certificate_parser::CertificateParser;
use certificate_service::infrastructure::apis;
use chrono::Utc;
use std::sync::Arc;
use tower::ServiceExt;

fn make_use_cases_with_mock() -> Arc<UseCases> {
    let mut mock_repo = MockCertificateRepositoryAbstract::new();
    mock_repo
        .expect_get_by_id()
        .returning(|id: &str| {
            let cert = Certificate::new(
                id.to_string(),
                "test.example.com".to_string(),
                "Test CA".to_string(),
                None,
                Utc::now() + chrono::Duration::days(90),
                None,
                vec![],
                None,
            );
            Ok(Some(cert))
        });
    mock_repo.expect_get_pem_by_id().returning(|_| Ok(None));
    mock_repo
        .expect_list_paginated()
        .returning(|page: u32, limit: u32, _status, _sort_by: CertificateSortBy, _sort_order: CertificateSortOrder| {
            let items = (0..limit.min(2))
                .map(|i| {
                    Certificate::new(
                        format!("id-{}", page + i),
                        format!("subject-{}", i),
                        "CA".to_string(),
                        None,
                        Utc::now() + chrono::Duration::days(30),
                        None,
                        vec![],
                        None,
                    )
                })
                .collect();
            Ok((items, 2))
        });
    mock_repo.expect_count_total().returning(|| Ok(2));
    mock_repo.expect_count_expiring_soon().returning(|_| Ok(1));
    mock_repo.expect_get_metrics().returning(|_| Ok((2, 1)));

    let repo = Arc::new(Box::new(mock_repo) as Box<dyn certificate_service::application::repositories::certificate_repository_abstract::CertificateRepositoryAbstract>);
    let parser: Arc<Box<dyn CertificateParser>> =
        Arc::new(Box::new(certificate_service::application::utils::certificate_parser_impl::X509CertificateParser::default()));
    Arc::new(UseCases::new(repo, parser))
}

#[tokio::test]
async fn get_certificates_list_returns_200() {
    let use_cases = make_use_cases_with_mock();
    let app = apis::create_router(use_cases);
    let req = Request::builder()
        .method("GET")
        .uri("/certificates?page=1&limit=10")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn get_certificates_by_id_returns_200_when_found() {
    let use_cases = make_use_cases_with_mock();
    let app = apis::create_router(use_cases);
    let req = Request::builder()
        .method("GET")
        .uri("/certificates/some-id")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn post_certificates_with_invalid_pem_returns_400() {
    let use_cases = make_use_cases_with_mock();
    let app = apis::create_router(use_cases);
    let body = serde_json::json!({ "pem": "not valid pem" });
    let req = Request::builder()
        .method("POST")
        .uri("/certificates")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
