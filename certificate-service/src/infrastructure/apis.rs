//! Axum router and HTTP wiring (driving adapter).
//! CORS layer handles OPTIONS preflight so browser POST requests (e.g. to /certificates) succeed.

use crate::adapters::inputs::http::certificate_controllers;
use crate::application::usecases::UseCases;
use axum::Router;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

pub fn create_router(use_cases: Arc<UseCases>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::PATCH,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([axum::http::header::CONTENT_TYPE, axum::http::header::AUTHORIZATION]);

    Router::new()
        .merge(certificate_controllers::certificate_routes())
        .layer(cors)
        .with_state(use_cases)
}
