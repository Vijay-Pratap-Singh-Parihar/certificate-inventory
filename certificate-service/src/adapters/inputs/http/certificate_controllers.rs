//! HTTP handlers: POST /certificates, GET /certificates (list), GET /certificates/metrics, GET /certificates/:id.
//! Maps JSON/query/path to UseCase input and ApplicationError to status/body.

use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::get_certificate::GetCertificateInput;
use crate::application::usecases::get_certificate_metrics::GetCertificateMetricsInput;
use crate::application::usecases::get_certificate_pem::GetCertificatePemInput;
use crate::application::usecases::interfaces::AbstractUseCase;
use crate::application::usecases::list_certificates::ListCertificatesInput;
use crate::application::usecases::save_certificate::SaveCertificateInput;
use crate::application::usecases::save_certificate_from_metadata::SaveCertificateFromMetadataInput;
use crate::application::usecases::UseCases;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;

pub fn certificate_routes() -> axum::Router<Arc<UseCases>> {
    axum::Router::new()
        .route("/certificates", axum::routing::post(save_certificate_handler).get(list_certificates_handler))
        .route("/certificates/metrics", axum::routing::get(metrics_handler))
        .route("/certificates/{id}/pem", axum::routing::get(get_certificate_pem_handler))
        .route("/certificates/{id}", axum::routing::get(get_certificate_handler))
}

/// Query parameters for GET /certificates (list with pagination, cursor, filter, sort).
#[derive(Debug, Deserialize, Default)]
pub struct ListCertificatesQueryParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
    /// Optional: "Valid", "Expiring Soon", "Expired"
    pub status: Option<String>,
    /// Optional: cursor for cursor-based pagination (certificate id; returns records after this id).
    pub cursor: Option<String>,
    /// Optional: sort_by = id | subject | issuer | expiration | last_updated. Default id.
    pub sort_by: Option<String>,
    /// Optional: sort_order = asc | desc. Default asc.
    pub sort_order: Option<String>,
}

fn default_page() -> u32 {
    1
}
fn default_limit() -> u32 {
    10
}

/// Request body: either { "pem": "..." } or { "subject", "issuer", "expiration", "san_entries" }.
#[derive(Debug, Deserialize)]
pub struct SaveCertificateRequest {
    pub pem: Option<String>,
    pub subject: Option<String>,
    pub issuer: Option<String>,
    pub expiration: Option<String>,
    #[serde(default)]
    pub san_entries: Option<Vec<String>>,
}

async fn save_certificate_handler(
    State(state): State<Arc<UseCases>>,
    Json(body): Json<SaveCertificateRequest>,
) -> impl IntoResponse {
    if let Some(pem) = body.pem.filter(|s| !s.trim().is_empty()) {
        let input = SaveCertificateInput { pem };
        match state.save_certificate.execute(Some(input)).await {
            Ok(out) => return (StatusCode::CREATED, Json(serde_json::json!(out.certificate))).into_response(),
            Err(e) => return map_application_error(e).into_response(),
        }
    }
    if let (Some(subject), Some(issuer), Some(expiration)) =
        (body.subject, body.issuer, body.expiration)
    {
        let input = SaveCertificateFromMetadataInput {
            subject,
            issuer,
            expiration,
            san_entries: body.san_entries.unwrap_or_default(),
        };
        match state.save_certificate_from_metadata.execute(Some(input)).await {
            Ok(out) => return (StatusCode::CREATED, Json(serde_json::json!(out.certificate))).into_response(),
            Err(e) => return map_application_error(e).into_response(),
        }
    }
    let err = crate::application::errors::application_error::ApplicationError::new(
        "Either 'pem' or 'subject', 'issuer', and 'expiration' are required".to_string(),
        "Validation",
        Some(400),
        None,
        None,
    );
    map_application_error(err).into_response()
}

async fn list_certificates_handler(
    State(state): State<Arc<UseCases>>,
    Query(params): Query<ListCertificatesQueryParams>,
) -> impl IntoResponse {
    let input = ListCertificatesInput {
        page: params.page,
        limit: params.limit,
        status: params.status,
        cursor: params.cursor,
        sort_by: params.sort_by,
        sort_order: params.sort_order,
    };
    match state.list_certificates.execute(Some(input)).await {
        Ok(out) => {
            if let Some(paginated) = out.paginated {
                (StatusCode::OK, Json(serde_json::json!(paginated))).into_response()
            } else {
                let cursor_res = out.cursor_response.expect("cursor_response set when paginated is None");
                (StatusCode::OK, Json(serde_json::json!(cursor_res))).into_response()
            }
        }
        Err(e) => map_application_error(e).into_response(),
    }
}

async fn metrics_handler(
    State(state): State<Arc<UseCases>>,
) -> impl IntoResponse {
    match state
        .get_certificate_metrics
        .execute(Some(GetCertificateMetricsInput))
        .await
    {
        Ok(out) => (StatusCode::OK, Json(out.metrics)).into_response(),
        Err(e) => map_application_error(e).into_response(),
    }
}

async fn get_certificate_handler(
    State(state): State<Arc<UseCases>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let input = GetCertificateInput { id };
    match state.get_certificate.execute(Some(input)).await {
        Ok(out) => (StatusCode::OK, Json(serde_json::json!(out.certificate))).into_response(),
        Err(e) => map_application_error(e).into_response(),
    }
}

async fn get_certificate_pem_handler(
    State(state): State<Arc<UseCases>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let input = GetCertificatePemInput { id };
    match state.get_certificate_pem.execute(Some(input)).await {
        Ok(out) => (StatusCode::OK, Json(serde_json::json!({ "pem": out.pem }))).into_response(),
        Err(e) => map_application_error(e).into_response(),
    }
}

fn map_application_error(e: ApplicationError) -> (StatusCode, Json<serde_json::Value>) {
    let status = match e.error_code() {
        Some(400) => StatusCode::BAD_REQUEST,
        Some(404) => StatusCode::NOT_FOUND,
        Some(500) => StatusCode::INTERNAL_SERVER_ERROR,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    let body = serde_json::json!({
        "error": e.message(),
        "code": e.error_code(),
    });
    (status, Json(body))
}
