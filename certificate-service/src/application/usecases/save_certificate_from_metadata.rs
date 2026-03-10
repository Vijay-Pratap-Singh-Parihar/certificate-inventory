//! Save certificate from raw metadata (subject, issuer, expiration, san_entries) without PEM.

use crate::application::errors::application_error::ApplicationError;
use crate::application::repositories::certificate_repository_abstract::CertificateRepositoryAbstract;
use crate::application::usecases::interfaces::AbstractUseCase;
use crate::domain::certificate::Certificate;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

pub struct SaveCertificateFromMetadataUseCase {
    certificate_repository: Arc<Box<dyn CertificateRepositoryAbstract>>,
}

impl SaveCertificateFromMetadataUseCase {
    pub fn new(certificate_repository: Arc<Box<dyn CertificateRepositoryAbstract>>) -> Self {
        Self {
            certificate_repository,
        }
    }
}

#[derive(Debug)]
pub struct SaveCertificateFromMetadataInput {
    pub subject: String,
    pub issuer: String,
    /// ISO 8601 datetime (e.g. "2025-12-31T23:59:59Z").
    pub expiration: String,
    pub san_entries: Vec<String>,
}

#[derive(Debug)]
pub struct SaveCertificateFromMetadataOutput {
    pub certificate: Certificate,
}

#[async_trait]
impl AbstractUseCase<SaveCertificateFromMetadataOutput, SaveCertificateFromMetadataInput>
    for SaveCertificateFromMetadataUseCase
{
    async fn execute(
        &self,
        input: Option<SaveCertificateFromMetadataInput>,
    ) -> Result<SaveCertificateFromMetadataOutput, ApplicationError> {
        let input = input.ok_or_else(|| {
            ApplicationError::into::<std::io::Error>(
                None,
                Some("Input is required".to_string()),
                None,
                None,
            )
        })?;

        let expiration: DateTime<Utc> = chrono::DateTime::parse_from_rfc3339(&input.expiration)
            .map_err(|e| {
                ApplicationError::into::<chrono::ParseError>(
                    Some(e),
                    Some("Invalid expiration date (use ISO 8601)".to_string()),
                    Some(400),
                    None,
                )
            })?
            .with_timezone(&Utc);

        let id = Uuid::new_v4().to_string();
        let certificate = Certificate::new(
            id.clone(),
            input.subject,
            input.issuer,
            None,
            expiration,
            None,
            input.san_entries,
            None,
        );

        info!(certificate_id = %certificate.id, "saving certificate from metadata");
        let saved = self
            .certificate_repository
            .save(certificate.clone(), "")
            .await?;
        Ok(SaveCertificateFromMetadataOutput {
            certificate: saved,
        })
    }
}
