//! SQLx/Postgres implementation of CertificateRepositoryAbstract.
//! Uses runtime queries (migration applied at startup in Step 4).

use crate::application::errors::application_error::ApplicationError;
use crate::application::repositories::certificate_repository_abstract::{
    CertificateRepositoryAbstract, CertificateStatusFilter, CertificateSortBy, CertificateSortOrder,
};
use crate::domain::certificate::Certificate;
use async_trait::async_trait;
use sqlx::FromRow;
use sqlx::PgPool;
use std::sync::Arc;

pub struct PostgresCertificateRepository {
    pool: Arc<PgPool>,
}

impl PostgresCertificateRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct CertificateRow {
    id: String,
    subject: String,
    issuer: String,
    expiration: chrono::DateTime<chrono::Utc>,
    san_entries: Vec<String>,
    last_updated: Option<chrono::DateTime<chrono::Utc>>,
}

#[async_trait]
impl CertificateRepositoryAbstract for PostgresCertificateRepository {
    async fn save(&self, certificate: Certificate, pem: &str) -> Result<Certificate, ApplicationError> {
        let row: CertificateRow = sqlx::query_as(
            r#"
            INSERT INTO certificates (id, subject, issuer, expiration, san_entries, pem)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, subject, issuer, expiration, san_entries, last_updated
            "#,
        )
        .bind(&certificate.id)
        .bind(&certificate.subject)
        .bind(&certificate.issuer)
        .bind(certificate.expiration)
        .bind(&certificate.san_entries)
        .bind(pem)
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| ApplicationError::into(Some(e), Some("Failed to save certificate"), Some(500), None))?;
        Ok(row_to_certificate(row))
    }

    async fn get_pem_by_id(&self, id: &str) -> Result<Option<String>, ApplicationError> {
        let row: Option<(Option<String>,)> = sqlx::query_as("SELECT pem FROM certificates WHERE id = $1")
            .bind(id)
            .fetch_optional(self.pool.as_ref())
            .await
            .map_err(|e| ApplicationError::into(Some(e), Some("Failed to get certificate PEM"), Some(500), None))?;
        Ok(row.and_then(|(pem,)| pem).and_then(|s| if s.is_empty() { None } else { Some(s) }))
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<Certificate>, ApplicationError> {
        let row: Option<CertificateRow> = sqlx::query_as(
            "SELECT id, subject, issuer, expiration, san_entries, last_updated FROM certificates WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| ApplicationError::into(Some(e), Some("Failed to get certificate"), Some(500), None))?;

        Ok(row.map(row_to_certificate))
    }

    async fn list_paginated(
        &self,
        page: u32,
        limit: u32,
        status: Option<CertificateStatusFilter>,
        sort_by: CertificateSortBy,
        sort_order: CertificateSortOrder,
    ) -> Result<(Vec<Certificate>, u64), ApplicationError> {
        let (where_clause, _) = status_clause(status);
        let count_sql = format!(
            "SELECT COUNT(*) as count FROM certificates {}",
            if where_clause.is_empty() {
                "".to_string()
            } else {
                format!("WHERE {}", where_clause)
            }
        );
        let total: (i64,) = sqlx::query_as(count_sql.as_str())
            .fetch_one(self.pool.as_ref())
            .await
            .map_err(|e| ApplicationError::into(Some(e), Some("Failed to count certificates"), Some(500), None))?;
        let total = total.0 as u64;

        let (order_col, order_dir) = order_clause(sort_by, sort_order);
        let (where_clause, _) = status_clause(status);
        let offset = (page.saturating_sub(1)) as u64 * (limit as u64);
        let order_limit = format!("ORDER BY {} {} LIMIT $1 OFFSET $2", order_col, order_dir);
        let select_sql = if where_clause.is_empty() {
            format!(
                "SELECT id, subject, issuer, expiration, san_entries, last_updated FROM certificates {}",
                order_limit
            )
        } else {
            format!(
                "SELECT id, subject, issuer, expiration, san_entries, last_updated FROM certificates WHERE {} {}",
                where_clause,
                order_limit
            )
        };

        let rows: Vec<CertificateRow> = sqlx::query_as(select_sql.as_str())
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(|e| ApplicationError::into(Some(e), Some("Failed to list certificates"), Some(500), None))?;

        let items = rows.into_iter().map(row_to_certificate).collect();
        Ok((items, total))
    }

    async fn list_cursor(
        &self,
        cursor_after_id: Option<&str>,
        limit: u32,
        status: Option<CertificateStatusFilter>,
    ) -> Result<(Vec<Certificate>, Option<String>), ApplicationError> {
        let limit_plus_one = limit.saturating_add(1) as i64;
        let (where_clause, _) = status_clause(status);
        let mut where_parts: Vec<String> = Vec::new();
        if !where_clause.is_empty() {
            where_parts.push(where_clause);
        }
        let (select_sql, bind_cursor_first) = if cursor_after_id.is_some() {
            where_parts.push("id > $1".to_string());
            let where_sql = format!("WHERE {}", where_parts.join(" AND "));
            (
                format!(
                    "SELECT id, subject, issuer, expiration, san_entries, last_updated FROM certificates {} ORDER BY id ASC LIMIT $2",
                    where_sql
                ),
                true,
            )
        } else {
            let where_sql = if where_parts.is_empty() {
                String::new()
            } else {
                format!("WHERE {}", where_parts.join(" AND "))
            };
            (
                format!(
                    "SELECT id, subject, issuer, expiration, san_entries, last_updated FROM certificates {} ORDER BY id ASC LIMIT $1",
                    where_sql
                ),
                false,
            )
        };
        let rows: Vec<CertificateRow> = if bind_cursor_first {
            sqlx::query_as(&select_sql)
                .bind(cursor_after_id.unwrap())
                .bind(limit_plus_one)
                .fetch_all(self.pool.as_ref())
                .await
        } else {
            sqlx::query_as(&select_sql)
                .bind(limit_plus_one)
                .fetch_all(self.pool.as_ref())
                .await
        }
        .map_err(|e| ApplicationError::into(Some(e), Some("Failed to list certificates by cursor"), Some(500), None))?;

        let has_more = rows.len() as i64 > limit as i64;
        let take = if has_more { limit as usize } else { rows.len() };
        let items: Vec<Certificate> = rows.into_iter().take(take).map(row_to_certificate).collect();
        let next_cursor = has_more.then(|| items.last().map(|c| c.id.clone())).flatten();
        Ok((items, next_cursor))
    }

    async fn count_total(&self) -> Result<u64, ApplicationError> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) as count FROM certificates")
            .fetch_one(self.pool.as_ref())
            .await
            .map_err(|e| ApplicationError::into(Some(e), Some("Failed to count certificates"), Some(500), None))?;
        Ok(row.0 as u64)
    }

    async fn count_expiring_soon(&self, within_days: u32) -> Result<u64, ApplicationError> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) as count FROM certificates WHERE expiration >= NOW() AND expiration <= NOW() + INTERVAL '1 day' * $1",
        )
        .bind(within_days as i32)
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| ApplicationError::into(Some(e), Some("Failed to count expiring certificates"), Some(500), None))?;
        Ok(row.0 as u64)
    }

    async fn get_metrics(&self, expiring_soon_days: u32) -> Result<(u64, u64), ApplicationError> {
        #[derive(sqlx::FromRow)]
        struct MetricsRow {
            total: i64,
            expiring_soon: i64,
        }
        let row: MetricsRow = sqlx::query_as(
            r#"
            SELECT
                COUNT(*)::bigint AS total,
                COUNT(*) FILTER (
                    WHERE expiration > NOW() AND expiration <= NOW() + INTERVAL '1 day' * $1
                )::bigint AS expiring_soon
            FROM certificates
            "#,
        )
        .bind(expiring_soon_days as i32)
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| ApplicationError::into(Some(e), Some("Failed to get metrics"), Some(500), None))?;
        Ok((row.total as u64, row.expiring_soon as u64))
    }
}

fn row_to_certificate(row: CertificateRow) -> Certificate {
    // valid_from and signature_algorithm are not stored in DB; enriched from PEM on GET by id when needed.
    Certificate::new(
        row.id,
        row.subject,
        row.issuer,
        None,
        row.expiration,
        None,
        row.san_entries,
        row.last_updated,
    )
}

/// Returns (WHERE clause fragment, number of bind params). Uses NOW() so no extra params.
fn status_clause(status: Option<CertificateStatusFilter>) -> (String, usize) {
    let s = match status {
        None => return (String::new(), 0),
        Some(CertificateStatusFilter::Expired) => "expiration < NOW()",
        Some(CertificateStatusFilter::ExpiringSoon) => {
            "expiration >= NOW() AND expiration <= NOW() + INTERVAL '30 days'"
        }
        Some(CertificateStatusFilter::Valid) => {
            "expiration > NOW() + INTERVAL '30 days'"
        }
    };
    (s.to_string(), 0)
}

/// Map sort enum to SQL column name (no user input; prevents SQL injection).
fn order_clause(sort_by: CertificateSortBy, sort_order: CertificateSortOrder) -> (&'static str, &'static str) {
    let col = match sort_by {
        CertificateSortBy::Id => "id",
        CertificateSortBy::Subject => "subject",
        CertificateSortBy::Issuer => "issuer",
        CertificateSortBy::Expiration => "expiration",
        CertificateSortBy::LastUpdated => "last_updated",
    };
    let dir = match sort_order {
        CertificateSortOrder::Asc => "ASC",
        CertificateSortOrder::Desc => "DESC",
    };
    (col, dir)
}
