use async_trait::async_trait;
use crate::application::errors::application_error::ApplicationError;
use crate::domain::certificate::Certificate;

#[cfg(test)]
use mockall::automock;

/// Filter for list endpoint: "Valid", "Expiring Soon", "Expired".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CertificateStatusFilter {
    Valid,
    ExpiringSoon,
    Expired,
}

/// Allowed sort columns for list (maps to SQL column names safely).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CertificateSortBy {
    Id,
    Subject,
    Issuer,
    Expiration,
    LastUpdated,
}

/// Sort direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CertificateSortOrder {
    Asc,
    Desc,
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait CertificateRepositoryAbstract: Send + Sync {
    /// Save certificate and its raw PEM. PEM is stored for later retrieval via get_pem_by_id.
    async fn save(&self, certificate: Certificate, pem: &str) -> Result<Certificate, ApplicationError>;
    async fn get_by_id(&self, id: &str) -> Result<Option<Certificate>, ApplicationError>;

    /// Return the raw PEM for a certificate by id. Returns None if not found or PEM not stored.
    async fn get_pem_by_id(&self, id: &str) -> Result<Option<String>, ApplicationError>;

    /// List certificates with offset pagination, optional status filter, and sorting.
    /// Returns (items for this page, total count matching filter).
    async fn list_paginated(
        &self,
        page: u32,
        limit: u32,
        status: Option<CertificateStatusFilter>,
        sort_by: CertificateSortBy,
        sort_order: CertificateSortOrder,
    ) -> Result<(Vec<Certificate>, u64), ApplicationError>;

    /// List certificates with cursor pagination (id > cursor, order by id ASC).
    /// Returns (items, next_cursor). next_cursor is Some(last_id) if there are more rows.
    async fn list_cursor(
        &self,
        cursor_after_id: Option<&str>,
        limit: u32,
        status: Option<CertificateStatusFilter>,
    ) -> Result<(Vec<Certificate>, Option<String>), ApplicationError>;

    /// Total count of all certificates.
    async fn count_total(&self) -> Result<u64, ApplicationError>;

    /// Count certificates expiring within the next `within_days` days (from now).
    async fn count_expiring_soon(&self, within_days: u32) -> Result<u64, ApplicationError>;

    /// Single query for dashboard metrics: (total, expiring_soon within 30 days).
    async fn get_metrics(&self, expiring_soon_days: u32) -> Result<(u64, u64), ApplicationError>;
}
