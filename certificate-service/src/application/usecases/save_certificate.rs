use crate::application::errors::application_error::ApplicationError;
use crate::application::repositories::certificate_repository_abstract::CertificateRepositoryAbstract;
use crate::application::usecases::interfaces::AbstractUseCase;
use crate::domain::certificate::Certificate;
use crate::domain::certificate_parser::CertificateParser;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

pub struct SaveCertificateUseCase {
    certificate_repository: Arc<Box<dyn CertificateRepositoryAbstract>>,
    certificate_parser: Arc<Box<dyn CertificateParser>>,
}

impl SaveCertificateUseCase {
    pub fn new(
        certificate_repository: Arc<Box<dyn CertificateRepositoryAbstract>>,
        certificate_parser: Arc<Box<dyn CertificateParser>>,
    ) -> Self {
        Self {
            certificate_repository,
            certificate_parser,
        }
    }
}

#[derive(Debug)]
pub struct SaveCertificateInput {
    /// PEM-encoded X.509 certificate string.
    pub pem: String,
}

#[derive(Debug)]
pub struct SaveCertificateOutput {
    pub certificate: Certificate,
}

#[async_trait]
impl AbstractUseCase<SaveCertificateOutput, SaveCertificateInput> for SaveCertificateUseCase {
    async fn execute(
        &self,
        input: Option<SaveCertificateInput>,
    ) -> Result<SaveCertificateOutput, ApplicationError> {
        let input = input.ok_or_else(|| {
            ApplicationError::into::<std::io::Error>(
                None,
                Some("Input is required".to_string()),
                None,
                None,
            )
        })?;

        let parsed = self.certificate_parser.parse_pem(&input.pem).map_err(|e| {
            ApplicationError::into::<std::io::Error>(
                None,
                Some(e),
                Some(400),
                None,
            )
        })?;

        let id = Uuid::new_v4().to_string();
        let certificate = Certificate::new(
            id,
            parsed.subject,
            parsed.issuer,
            Some(parsed.valid_from),
            parsed.expiration,
            Some(parsed.signature_algorithm),
            parsed.san_entries,
            None, // DB sets last_updated on insert
        );

        info!(certificate_id = %certificate.id, "saving certificate");
        let saved = self.certificate_repository.save(certificate, &input.pem).await?;
        Ok(SaveCertificateOutput { certificate: saved })
    }
}
