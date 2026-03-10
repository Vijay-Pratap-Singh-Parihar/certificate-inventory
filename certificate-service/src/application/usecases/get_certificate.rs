use crate::application::errors::application_error::ApplicationError;
use crate::application::repositories::certificate_repository_abstract::CertificateRepositoryAbstract;
use crate::application::usecases::interfaces::AbstractUseCase;
use crate::domain::certificate_parser::CertificateParser;
use crate::domain::certificate::Certificate;
use async_trait::async_trait;
use std::sync::Arc;

pub struct GetCertificateUseCase {
    certificate_repository: Arc<Box<dyn CertificateRepositoryAbstract>>,
    certificate_parser: Arc<Box<dyn CertificateParser>>,
}

impl GetCertificateUseCase {
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
pub struct GetCertificateInput {
    pub id: String,
}

#[derive(Debug)]
pub struct GetCertificateOutput {
    pub certificate: Certificate,
}

#[async_trait]
impl AbstractUseCase<GetCertificateOutput, GetCertificateInput> for GetCertificateUseCase {
    async fn execute(
        &self,
        input: Option<GetCertificateInput>,
    ) -> Result<GetCertificateOutput, ApplicationError> {
        let input = input.ok_or_else(|| {
            ApplicationError::into::<std::io::Error>(
                None,
                Some("Input is required".to_string()),
                None,
                None,
            )
        })?;

        let mut certificate = self
            .certificate_repository
            .get_by_id(&input.id)
            .await?
            .ok_or_else(|| {
                ApplicationError::into::<std::io::Error>(
                    None,
                    Some(format!("Certificate with id {} not found", input.id)),
                    Some(404),
                    None,
                )
            })?;

        // Enrich valid_from and signature_algorithm from stored PEM (not persisted in DB columns).
        if let Ok(Some(pem)) = self.certificate_repository.get_pem_by_id(&input.id).await {
            if let Ok(parsed) = self.certificate_parser.parse_pem(&pem) {
                certificate.valid_from = Some(parsed.valid_from);
                certificate.signature_algorithm = Some(parsed.signature_algorithm);
            }
        }

        Ok(GetCertificateOutput { certificate })
    }
}
