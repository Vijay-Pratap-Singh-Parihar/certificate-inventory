use crate::application::dto::{CursorListResponse, PaginatedResponse};
use crate::application::errors::application_error::ApplicationError;
use crate::application::repositories::certificate_repository_abstract::{
    CertificateRepositoryAbstract, CertificateSortBy, CertificateSortOrder, CertificateStatusFilter,
};
use crate::application::usecases::interfaces::AbstractUseCase;
use crate::domain::certificate::Certificate;
use async_trait::async_trait;
use std::sync::Arc;

pub struct ListCertificatesUseCase {
    certificate_repository: Arc<Box<dyn CertificateRepositoryAbstract>>,
}

impl ListCertificatesUseCase {
    pub fn new(certificate_repository: Arc<Box<dyn CertificateRepositoryAbstract>>) -> Self {
        Self {
            certificate_repository,
        }
    }
}

#[derive(Debug)]
pub struct ListCertificatesInput {
    pub page: u32,
    pub limit: u32,
    /// Optional: "Valid", "Expiring Soon", "Expired"
    pub status: Option<String>,
    /// Optional: cursor for cursor-based pagination (id after which to fetch). When set, response is cursor shape.
    pub cursor: Option<String>,
    /// Optional: sort column (id, subject, issuer, expiration, last_updated). Default id.
    pub sort_by: Option<String>,
    /// Optional: sort order (asc, desc). Default asc.
    pub sort_order: Option<String>,
}

#[derive(Debug)]
pub struct ListCertificatesOutput {
    /// Set when using page/limit (no cursor).
    pub paginated: Option<PaginatedResponse<Certificate>>,
    /// Set when using cursor.
    pub cursor_response: Option<CursorListResponse<Certificate>>,
}

fn parse_status(s: &str) -> Option<CertificateStatusFilter> {
    match s.trim() {
        "Valid" => Some(CertificateStatusFilter::Valid),
        "Expiring Soon" => Some(CertificateStatusFilter::ExpiringSoon),
        "Expired" => Some(CertificateStatusFilter::Expired),
        _ => None,
    }
}

fn parse_sort_by(s: &str) -> CertificateSortBy {
    match s.trim().to_lowercase().as_str() {
        "subject" => CertificateSortBy::Subject,
        "issuer" => CertificateSortBy::Issuer,
        "expiration" => CertificateSortBy::Expiration,
        "last_updated" => CertificateSortBy::LastUpdated,
        _ => CertificateSortBy::Id,
    }
}

fn parse_sort_order(s: &str) -> CertificateSortOrder {
    match s.trim().to_lowercase().as_str() {
        "desc" => CertificateSortOrder::Desc,
        _ => CertificateSortOrder::Asc,
    }
}

#[async_trait]
impl AbstractUseCase<ListCertificatesOutput, ListCertificatesInput> for ListCertificatesUseCase {
    async fn execute(
        &self,
        input: Option<ListCertificatesInput>,
    ) -> Result<ListCertificatesOutput, ApplicationError> {
        let input = input.unwrap_or(ListCertificatesInput {
            page: 1,
            limit: 10,
            status: None,
            cursor: None,
            sort_by: None,
            sort_order: None,
        });

        let limit = input.limit.clamp(1, 10_000);
        let status = input.status.as_deref().and_then(parse_status);

        if let Some(cursor) = input.cursor.filter(|s| !s.trim().is_empty()) {
            let (data, next_cursor) = self
                .certificate_repository
                .list_cursor(Some(cursor.as_str()), limit, status)
                .await?;
            return Ok(ListCertificatesOutput {
                paginated: None,
                cursor_response: Some(CursorListResponse {
                    data,
                    next_cursor,
                }),
            });
        }

        let page = input.page.max(1);
        let sort_by = parse_sort_by(input.sort_by.as_deref().unwrap_or("id"));
        let sort_order = parse_sort_order(input.sort_order.as_deref().unwrap_or("asc"));

        let (items, total) = self
            .certificate_repository
            .list_paginated(page, limit, status, sort_by, sort_order)
            .await?;

        let paginated = PaginatedResponse::new(items, total, page, limit);
        Ok(ListCertificatesOutput {
            paginated: Some(paginated),
            cursor_response: None,
        })
    }
}
